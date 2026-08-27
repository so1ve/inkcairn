use std::fs;
use std::path::{Path, PathBuf};
use std::sync::{Arc, RwLock};
use std::time::Duration;

use anyhow::Result;
use axum::Router;
use axum::body::{Body, to_bytes};
use axum::extract::{Request, State};
use axum::http::header::{
    ACCEPT_RANGES, CACHE_CONTROL, CONTENT_LENGTH, ETAG, IF_MODIFIED_SINCE, IF_NONE_MATCH, IF_RANGE,
    LAST_MODIFIED, RANGE,
};
use axum::http::{HeaderValue, Method, StatusCode, Uri};
use axum::response::{IntoResponse, Response};
use axum::routing::get;
use get_port::tcp::TcpPort;
use get_port::{Ops, Range};
use notify::{Event as FileEvent, EventKind, RecursiveMode, Watcher};
use tokio::sync::mpsc;
use tower::ServiceExt;
use tower_http::services::ServeDir;

use crate::build;

const RELOAD_PATH: &str = "/_inkcairn/reload";
const HOST: &str = "127.0.0.1";
const DEFAULT_PORT: u16 = 3201;

#[derive(Clone)]
struct Preview {
    directory: PathBuf,
    state: Arc<RwLock<PreviewState>>,
}

struct PreviewState {
    base_path: String,
    revision: u64,
}

pub async fn run(requested_root: &Path, port: Option<u16>) -> Result<()> {
    let root = fs::canonicalize(requested_root)?;
    let temporary = tempfile::tempdir()?;
    let directory = temporary.path().join("site");
    let state = Arc::new(RwLock::new(PreviewState {
        base_path: build::preview(&root, &directory)?,
        revision: 0,
    }));
    let port = port
        .or_else(|| {
            TcpPort::in_range(
                HOST,
                Range {
                    min: DEFAULT_PORT,
                    max: u16::MAX,
                },
            )
        })
        .ok_or_else(|| anyhow::anyhow!("no available TCP port found"))?;
    let address = format!("{HOST}:{port}");
    let listener = tokio::net::TcpListener::bind(&address).await?;
    let (sender, events) = mpsc::unbounded_channel();
    let mut watcher = notify::recommended_watcher(move |event| {
        _ = sender.send(event);
    })?;
    watcher.watch(&root, RecursiveMode::Recursive)?;
    let preview = Preview {
        directory: directory.clone(),
        state: Arc::clone(&state),
    };
    let displayed_base = preview.state.read().unwrap().base_path.clone();
    let app = Router::new()
        .route(RELOAD_PATH, get(reload_revision))
        .fallback(files)
        .with_state(preview);

    println!(
        "Serving {} at http://{address}{displayed_base}/",
        root.display()
    );

    let result = tokio::select! {
        result = axum::serve(listener, app) => result,
        _ = tokio::signal::ctrl_c() => Ok(()),
        _ = rebuild_on_change(&root, &directory, state, events) => unreachable!(),
    };
    result?;

    Ok(())
}

async fn rebuild_on_change(
    root: &Path,
    directory: &Path,
    state: Arc<RwLock<PreviewState>>,
    mut events: mpsc::UnboundedReceiver<notify::Result<FileEvent>>,
) {
    loop {
        let mut rebuild = affects_site(root, events.recv().await.unwrap());

        while let Ok(event) = tokio::time::timeout(Duration::from_millis(100), events.recv()).await
        {
            rebuild |= affects_site(root, event.unwrap());
        }

        if rebuild {
            match build::preview(root, directory) {
                Ok(base) => {
                    let mut state = state.write().unwrap();
                    state.base_path = base;
                    state.revision += 1;
                    println!("Rebuilt site");
                }
                Err(error) => eprintln!("Build failed:\n{error:#}"),
            }
        }
    }
}

fn affects_site(root: &Path, event: notify::Result<FileEvent>) -> bool {
    let event = match event {
        Ok(event) => event,
        Err(error) => {
            eprintln!("Watch failed: {error}");

            return false;
        }
    };
    if matches!(event.kind, EventKind::Access(_)) {
        return false;
    }

    event.paths.iter().any(|path| {
        let Ok(relative) = path.strip_prefix(root) else {
            return false;
        };
        let Some(component) = relative.components().next() else {
            return false;
        };

        matches!(
            component.as_os_str().to_str(),
            Some("inkcairn.md" | "posts" | "pages" | "snippets" | "assets")
        )
    })
}

async fn reload_revision(State(preview): State<Preview>) -> String {
    preview.state.read().unwrap().revision.to_string()
}

async fn files(State(preview): State<Preview>, mut request: Request) -> Response {
    let (path, page_revision) = {
        let state = preview.state.read().unwrap();
        let Some(path) = strip_base(request.uri().path(), &state.base_path) else {
            return StatusCode::NOT_FOUND.into_response();
        };

        let path = if path == "/" {
            "/index.html".to_owned()
        } else {
            path.to_owned()
        };

        (path, state.revision)
    };
    let method = request.method().clone();
    let html = path.ends_with(".html") && !path.starts_with("/assets/");
    *request.uri_mut() = path.parse::<Uri>().unwrap();
    if html {
        request.headers_mut().remove(RANGE);
        request.headers_mut().remove(IF_RANGE);
        request.headers_mut().remove(IF_NONE_MATCH);
        request.headers_mut().remove(IF_MODIFIED_SINCE);
    }
    let mut response = ServeDir::new(preview.directory)
        .oneshot(request)
        .await
        .unwrap()
        .map(Body::new);
    response
        .headers_mut()
        .insert(CACHE_CONTROL, HeaderValue::from_static("no-store"));

    if response.status() != StatusCode::OK || !html {
        return response;
    }

    response.headers_mut().remove(ACCEPT_RANGES);
    response.headers_mut().remove(ETAG);
    response.headers_mut().remove(LAST_MODIFIED);
    let reload_script = reload_script(page_revision);

    if method == Method::HEAD {
        let length = response
            .headers()
            .get(CONTENT_LENGTH)
            .unwrap()
            .to_str()
            .unwrap()
            .parse::<usize>()
            .unwrap()
            + reload_script.len();
        response.headers_mut().insert(
            CONTENT_LENGTH,
            HeaderValue::from_str(&length.to_string()).unwrap(),
        );

        return response;
    }

    let (mut parts, body) = response.into_parts();
    let bytes = to_bytes(body, usize::MAX).await.unwrap();
    let mut html = String::from_utf8(bytes.to_vec()).unwrap();
    let closing_body = html.rfind("</body>").unwrap();
    html.insert_str(closing_body, &reload_script);
    parts.headers.insert(
        CONTENT_LENGTH,
        HeaderValue::from_str(&html.len().to_string()).unwrap(),
    );

    Response::from_parts(parts, Body::from(html))
}

fn reload_script(revision: u64) -> String {
    format!(
        r#"<script>{{const v={revision},check=async()=>{{try{{if(+await fetch("{RELOAD_PATH}",{{cache:"no-store"}}).then(r=>r.text())!==v)return location.reload()}}catch{{}}setTimeout(check,500)}};setTimeout(check,500)}}</script>"#
    )
}

fn strip_base<'a>(path: &'a str, base_path: &str) -> Option<&'a str> {
    if base_path.is_empty() {
        Some(path)
    } else if path == base_path {
        Some("/")
    } else {
        path.strip_prefix(base_path)
            .filter(|local| local.starts_with('/'))
    }
}
