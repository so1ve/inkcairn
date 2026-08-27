use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use time::{Date, OffsetDateTime};

use crate::git::GitIndex;
use crate::parser::Parser;
use crate::url_path;

mod filename;

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
    pub pinned: bool,
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
    let mut posts = filename::Posts::new();

    for source in files {
        let name = filename::post_name(&source)
            .with_context(|| format!("invalid post filename {}", source.display()))?;
        let document = parse_document(root, &source, parser, git, name.slug(), name.draft())?;
        let path = PostPath::from_source(&directory, &source, name.slug());

        if !outputs.insert(path.output.clone()) {
            bail!(
                "{} has duplicate post output `{}`",
                source.display(),
                path.output
            );
        }
        let published = document.published;
        let document_source = document.source.clone();
        posts.push(
            &document_source,
            published,
            &name,
            Post {
                document,
                path,
                pinned: name.pinned(),
            },
        )?;
    }

    Ok(posts
        .into_values()
        .into_iter()
        .filter(|post| include_drafts || !post.document.draft)
        .collect())
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
        let name = filename::page_name(&source);
        let document = parse_document(root, &source, parser, git, name.slug(), name.draft())?;
        let path = PagePath::from_slug(name.slug());

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
    fallback_title: &str,
    draft: bool,
) -> Result<Document> {
    let markdown = fs::read_to_string(source_path)?;
    let frontmatter = match parser.frontmatter(&markdown).as_deref() {
        Some("") | None => Frontmatter::default(),
        Some(frontmatter) => yaml_serde::from_str(frontmatter)?,
    };
    let (inferred_published, updated) = infer_dates(source_path, git)?;
    let published = frontmatter.published.unwrap_or(inferred_published);
    let source = source_path.strip_prefix(root).unwrap().to_owned();

    Ok(Document {
        source,
        fallback_title: fallback_title.to_owned(),
        draft,
        published,
        updated,
        markdown,
    })
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

#[cfg(test)]
mod tests {
    use tempfile::tempdir;

    use super::*;

    #[test]
    fn post_names_control_order_and_urls() {
        let directory = tempdir().unwrap();
        write_post(directory.path(), "02-second-pin.md", "2026-08-27");
        write_post(directory.path(), "00-first-pin.md", "2020-01-01");
        write_post(directory.path(), "2026-08-27-01-second.md", "2026-08-27");
        write_post(directory.path(), "2026-08-27-00-first.md", "2026-08-27");
        write_post(directory.path(), "2026-08-27-unordered.md", "2026-08-27");
        write_post(directory.path(), "plain.md", "2026-08-27");
        write_post(directory.path(), "2026-08-26-older.md", "2026-08-26");

        let content = discover(directory.path(), &Parser::new(), None, false).unwrap();
        let sources = content
            .posts
            .iter()
            .map(|post| post.document.source.to_str().unwrap())
            .collect::<Vec<_>>();

        assert_eq!(
            sources,
            [
                "posts/00-first-pin.md",
                "posts/02-second-pin.md",
                "posts/2026-08-27-00-first.md",
                "posts/2026-08-27-01-second.md",
                "posts/2026-08-27-unordered.md",
                "posts/plain.md",
                "posts/2026-08-26-older.md",
            ]
        );
        assert_eq!(content.posts[0].path.url, "posts/first-pin.html");
        assert_eq!(content.posts[2].path.url, "posts/first.html");
        assert_eq!(content.posts[4].path.url, "posts/unordered.html");
        assert_eq!(content.posts[0].document.fallback_title, "first-pin");
        assert!(content.posts[0].pinned);
        assert!(content.posts[1].pinned);
        assert!(!content.posts[2].pinned);
    }

    #[test]
    fn date_prefix_must_match_publication_date() {
        let directory = tempdir().unwrap();
        write_post(directory.path(), "2026-08-27-mismatch.md", "2026-08-26");

        let error = discover(directory.path(), &Parser::new(), None, false)
            .err()
            .unwrap()
            .to_string();

        assert!(error.contains("date prefix `2026-08-27`"));
        assert!(error.contains("publication date `2026-08-26`"));
    }

    #[test]
    fn positions_cannot_be_reused() {
        let directory = tempdir().unwrap();
        write_post(directory.path(), "00-first.md", "2026-08-27");
        write_post(directory.path(), "00-second.md", "2026-08-26");

        let error = discover(directory.path(), &Parser::new(), None, false)
            .err()
            .unwrap()
            .to_string();

        assert!(error.contains("same pinned position `00-`"));
    }

    fn write_post(root: &Path, name: &str, published: &str) {
        let posts = root.join("posts");
        fs::create_dir_all(&posts).unwrap();
        fs::write(
            posts.join(name),
            format!("---\npublished: {published}\n---\n\n# {name}\n"),
        )
        .unwrap();
    }
}
