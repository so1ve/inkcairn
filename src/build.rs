use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use rayon::prelude::*;

use crate::git::GitIndex;
use crate::metadata::Metadata;
use crate::output::{OutputFile, SiteOutput};
use crate::parser::Parser;
use crate::render::{RenderedPage, RenderedPost, Renderer, search_stylesheet, stylesheet};
use crate::{categories, content, search, templates, url_path};

pub fn build(root: &Path, allow_dirty: bool, include_drafts: bool) -> Result<PathBuf> {
    let root = fs::canonicalize(root)?;
    let destination = root.join("dist");
    generate(&root, &destination, allow_dirty, include_drafts)?;

    Ok(destination)
}

pub fn preview(root: &Path, destination: &Path) -> Result<String> {
    generate(root, destination, true, true)
}

fn generate(
    root: &Path,
    destination: &Path,
    allow_dirty: bool,
    include_drafts: bool,
) -> Result<String> {
    let parser = Parser::new();
    let metadata = Metadata::load(root, &parser)?;
    let git = GitIndex::discover(root)?;
    if !allow_dirty {
        match git.as_ref() {
            Some(git) if git.head.is_none() => {
                bail!("Git repository has no commits; pass --allow-dirty to build it")
            }
            Some(git) if git.dirty => {
                bail!("Git worktree has uncommitted changes; pass --allow-dirty to build it")
            }
            None => bail!("site is not in a Git repository; pass --allow-dirty to build it"),
            Some(_) => {}
        }
    }
    let content = content::discover(root, &parser, git.as_ref(), include_drafts)?;
    let posts = content.posts;
    let mut pages = content.pages;

    pages.sort_by(|left, right| left.document.source.cmp(&right.document.source));

    let renderer = Renderer::new(parser);
    let posts = posts
        .into_par_iter()
        .map(|post| renderer.post(post))
        .collect::<Result<Vec<_>>>()?;
    let pages = pages
        .into_par_iter()
        .map(|page| renderer.page(page))
        .collect::<Result<Vec<_>>>()?;
    let categories = categories::collect(&posts);
    let snippets = load_snippets(root, &renderer)?;
    let template_build = if let Some(git) = git.as_ref()
        && let Some(head) = git.head.as_deref()
    {
        let short_head = &head[..7];

        Some(templates::Build {
            label: if git.dirty {
                format!("{short_head} (dirty)")
            } else {
                short_head.to_owned()
            },
            url: git
                .origin
                .as_ref()
                .map(|origin| format!("{origin}/commit/{head}")),
        })
    } else {
        None
    };
    let site = templates::Site::new(
        &metadata,
        template_build,
        &pages,
        templates::Snippets {
            head: snippets.head.as_deref(),
            home: snippets.home.as_deref(),
            after_content: snippets.after_content.as_deref(),
        },
    );
    let mut files = posts
        .par_iter()
        .map(|post| render_post(post, &site))
        .collect::<Vec<_>>();
    files.par_extend(pages.par_iter().map(|page| render_page(page, &site)));

    files.push(OutputFile {
        path: PathBuf::from("index.html"),
        bytes: site.home(&posts).into_bytes(),
        source: None,
    });
    files.push(OutputFile {
        path: PathBuf::from("archive.html"),
        bytes: site.archive(&posts).into_bytes(),
        source: None,
    });
    files.push(OutputFile {
        path: PathBuf::from("posts.html"),
        bytes: site.posts(&categories, &posts).into_bytes(),
        source: None,
    });
    render_categories(&mut files, &categories, &posts, &site);
    files.push(OutputFile {
        path: PathBuf::from("style.css"),
        bytes: stylesheet().into_bytes(),
        source: None,
    });
    files.push(OutputFile {
        path: PathBuf::from("script.js"),
        bytes: include_bytes!("../theme/script.js").to_vec(),
        source: None,
    });
    // search
    files.push(OutputFile {
        path: PathBuf::from("search.html"),
        bytes: site.search_page().into_bytes(),
        source: None,
    });
    files.push(OutputFile {
        path: PathBuf::from("search.js"),
        bytes: include_bytes!("../theme/search.js").to_vec(),
        source: None,
    });
    files.push(OutputFile {
        path: PathBuf::from("minisearch.js"),
        bytes: include_bytes!("../theme/minisearch.js").to_vec(),
        source: None,
    });
    files.push(OutputFile {
        path: PathBuf::from("search.css"),
        bytes: search_stylesheet().into_bytes(),
        source: None,
    });
    files.push(search::documents(&posts, &pages));

    let mut output = SiteOutput::new(root, files, git.as_ref());
    if metadata.url.is_some() {
        output.push(OutputFile {
            path: PathBuf::from("rss.xml"),
            bytes: site.feed(&posts).into_bytes(),
            source: None,
        });
        let paths = output.html_paths();
        output.push(OutputFile {
            path: PathBuf::from("sitemap.xml"),
            bytes: site.sitemap(&paths).into_bytes(),
            source: None,
        });
    }
    output.push(OutputFile {
        path: PathBuf::from("404.html"),
        bytes: site.not_found().into_bytes(),
        source: None,
    });

    output.write(destination)?;

    Ok(url_path::base(metadata.url.as_deref()))
}

struct Snippets {
    head: Option<String>,
    home: Option<String>,
    after_content: Option<String>,
}

fn load_snippets(root: &Path, renderer: &Renderer) -> Result<Snippets> {
    Ok(Snippets {
        head: load_snippet(root, "head", renderer)?,
        home: load_snippet(root, "home", renderer)?,
        after_content: load_snippet(root, "after-content", renderer)?,
    })
}

fn load_snippet(root: &Path, name: &str, renderer: &Renderer) -> Result<Option<String>> {
    let directory = root.join("snippets");
    let markdown = read_optional(&directory.join(format!("{name}.md")))?;
    let html = read_optional(&directory.join(format!("{name}.html")))?;

    match (markdown, html) {
        (Some(_), Some(_)) => bail!("snippets/{name}.md and snippets/{name}.html both exist"),
        (Some(markdown), None) => Ok(Some(renderer.markdown(&markdown)?)),
        (None, html) => Ok(html),
    }
}

fn read_optional(path: &Path) -> Result<Option<String>> {
    match fs::read_to_string(path) {
        Ok(value) => Ok(Some(value)),
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => Ok(None),
        Err(error) => Err(error.into()),
    }
}

fn render_post(post: &RenderedPost, site: &templates::Site<'_>) -> OutputFile {
    OutputFile {
        path: PathBuf::from(&post.path.output),
        bytes: site.post_article(post).into_bytes(),
        source: Some(post.article.source.clone()),
    }
}

fn render_page(page: &RenderedPage, site: &templates::Site<'_>) -> OutputFile {
    OutputFile {
        path: PathBuf::from(&page.path.output),
        bytes: site.page_article(page).into_bytes(),
        source: Some(page.article.source.clone()),
    }
}

fn render_categories(
    files: &mut Vec<OutputFile>,
    categories: &[categories::Category],
    posts: &[RenderedPost],
    site: &templates::Site<'_>,
) {
    for category in categories {
        let current = category.path.last().unwrap();

        files.push(OutputFile {
            path: PathBuf::from(&current.output),
            bytes: site.category(category, posts).into_bytes(),
            source: None,
        });
        render_categories(files, &category.children, posts, site);
    }
}
