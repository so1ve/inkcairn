use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Result, bail};
use serde::Deserialize;
use time::{Date, OffsetDateTime};

use crate::git::GitIndex;
use crate::parser::Parser;
use crate::url_path;

#[derive(Clone)]
pub struct CategoryPath {
    pub label: String,
    pub output: String,
    pub url: String,
}

pub struct PostPath {
    pub output: String,
    pub url: String,
    pub categories: Vec<CategoryPath>,
}

pub struct PagePath {
    pub output: String,
    pub url: String,
}

pub struct Document {
    pub source: PathBuf,
    pub fallback_title: String,
    pub draft: bool,
    pub published: Date,
    pub updated: Date,
    pub markdown: String,
}

pub struct Post {
    pub document: Document,
    pub path: PostPath,
}

pub struct Page {
    pub document: Document,
    pub path: PagePath,
}

pub struct Content {
    pub posts: Vec<Post>,
    pub pages: Vec<Page>,
}

impl PostPath {
    fn from_source(directory: &Path, source: &Path, slug: &str) -> Self {
        let mut labels = Vec::new();
        let categories = source
            .parent()
            .unwrap()
            .strip_prefix(directory)
            .unwrap()
            .components()
            .map(|component| {
                let label = component.as_os_str().to_str().unwrap().replace("__", " ");
                labels.push(label.clone());

                let output = format!("posts/{}.html", labels.join("/"));

                CategoryPath {
                    label,
                    url: url_path::encode(&output),
                    output,
                }
            })
            .collect::<Vec<_>>();
        let output = if labels.is_empty() {
            format!("posts/{slug}.html")
        } else {
            format!("posts/{}/{slug}.html", labels.join("/"))
        };

        Self {
            url: url_path::encode(&output),
            output,
            categories,
        }
    }
}

impl PagePath {
    fn from_slug(slug: &str) -> Self {
        let output = format!("{slug}.html");

        Self {
            url: url_path::encode(&output),
            output,
        }
    }
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct Frontmatter {
    published: Option<Date>,
}

pub fn discover(
    root: &Path,
    parser: &Parser,
    git: Option<&GitIndex>,
    include_drafts: bool,
) -> Result<Content> {
    Ok(Content {
        posts: discover_posts(root, parser, git, include_drafts)?,
        pages: discover_pages(root, parser, git, include_drafts)?,
    })
}

fn discover_posts(
    root: &Path,
    parser: &Parser,
    git: Option<&GitIndex>,
    include_drafts: bool,
) -> Result<Vec<Post>> {
    let directory = root.join("posts");
    let mut files = Vec::new();
    collect_markdown(&directory, &mut files)?;
    files.sort();

    let mut outputs = HashSet::new();
    let mut posts = Vec::new();

    for source in files {
        let (document, slug) = parse_document(root, &source, parser, git)?;
        let path = PostPath::from_source(&directory, &source, &slug);

        if !outputs.insert(path.output.clone()) {
            bail!(
                "{} has duplicate post output `{}`",
                source.display(),
                path.output
            );
        }
        if include_drafts || !document.draft {
            posts.push(Post { document, path });
        }
    }

    Ok(posts)
}

fn discover_pages(
    root: &Path,
    parser: &Parser,
    git: Option<&GitIndex>,
    include_drafts: bool,
) -> Result<Vec<Page>> {
    let directory = root.join("pages");
    let mut files = Vec::new();
    collect_markdown(&directory, &mut files)?;
    files.sort();

    let mut outputs = HashSet::new();
    let mut pages = Vec::new();

    for source in files {
        let (document, slug) = parse_document(root, &source, parser, git)?;
        let path = PagePath::from_slug(&slug);

        if !outputs.insert(path.output.clone()) {
            bail!(
                "{} has duplicate page output `{}`",
                source.display(),
                path.output
            );
        }
        if include_drafts || !document.draft {
            pages.push(Page { document, path });
        }
    }

    Ok(pages)
}

fn parse_document(
    root: &Path,
    source_path: &Path,
    parser: &Parser,
    git: Option<&GitIndex>,
) -> Result<(Document, String)> {
    let file_stem = source_path.file_stem().unwrap().to_str().unwrap();
    let (file_stem, draft) = match file_stem.strip_suffix(".draft") {
        Some(file_stem) => (file_stem, true),
        None => (file_stem, false),
    };
    let markdown = fs::read_to_string(source_path)?;
    let frontmatter = match parser.frontmatter(&markdown).as_deref() {
        Some("") | None => Frontmatter::default(),
        Some(frontmatter) => yaml_serde::from_str(frontmatter)?,
    };
    let (inferred_published, updated) = infer_dates(source_path, git)?;
    let published = frontmatter.published.unwrap_or(inferred_published);
    let source = source_path.strip_prefix(root).unwrap().to_owned();

    Ok((
        Document {
            source,
            fallback_title: file_stem.to_owned(),
            draft,
            published,
            updated,
            markdown,
        },
        strip_order_prefix(file_stem).to_owned(),
    ))
}

fn collect_markdown(directory: &Path, output: &mut Vec<PathBuf>) -> Result<()> {
    let entries = match fs::read_dir(directory) {
        Ok(entries) => entries,
        Err(error) if error.kind() == std::io::ErrorKind::NotFound => return Ok(()),
        Err(error) => return Err(error.into()),
    };

    for entry in entries {
        let entry = entry?;
        let path = entry.path();
        let file_type = entry.file_type()?;
        if file_type.is_dir() {
            collect_markdown(&path, output)?;
        } else if path.extension().unwrap() == "md" {
            output.push(path);
        }
    }

    Ok(())
}

fn infer_dates(path: &Path, git: Option<&GitIndex>) -> Result<(Date, Date)> {
    let history = match git {
        Some(git) => git.file_info(path)?,
        None => None,
    };

    match history {
        Some(history) if history.dirty => Ok((
            history.created_at,
            OffsetDateTime::from(fs::metadata(path)?.modified()?).date(),
        )),
        Some(history) => Ok((history.created_at, history.updated_at)),
        None => filesystem_dates(path),
    }
}

fn filesystem_dates(path: &Path) -> Result<(Date, Date)> {
    let metadata = fs::metadata(path)?;
    let updated = metadata.modified()?;
    let created = match metadata.created() {
        Ok(created) => created,
        Err(_) => updated,
    };

    Ok((
        OffsetDateTime::from(created).date(),
        OffsetDateTime::from(updated).date(),
    ))
}

fn strip_order_prefix(stem: &str) -> &str {
    let bytes = stem.as_bytes();
    if bytes.len() > 3 && bytes[0].is_ascii_digit() && bytes[1].is_ascii_digit() && bytes[2] == b'-'
    {
        return &stem[3..];
    }

    stem
}
