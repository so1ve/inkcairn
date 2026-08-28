use std::collections::HashSet;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::SystemTime;

use anyhow::{Context, Result, bail};
use serde::Deserialize;
use serde::de::DeserializeOwned;
use time::OffsetDateTime;

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

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Repost {
    pub url: Option<String>,
    pub title: Option<String>,
    pub author: String,
    #[serde(default, with = "crate::date_time::option")]
    pub published: Option<OffsetDateTime>,
}

pub struct Document {
    pub source: PathBuf,
    pub fallback_title: String,
    pub draft: bool,
    pub published: OffsetDateTime,
    pub updated: OffsetDateTime,
    pub markdown: String,
}

pub struct Post {
    pub document: Document,
    pub path: PostPath,
    pub pinned: bool,
    pub repost: Option<Repost>,
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
struct PostFrontmatter {
    #[serde(default, with = "crate::date_time::option")]
    published: Option<OffsetDateTime>,
    repost: Option<Repost>,
}

#[derive(Default, Deserialize)]
#[serde(deny_unknown_fields)]
struct PageFrontmatter {
    #[serde(default, with = "crate::date_time::option")]
    published: Option<OffsetDateTime>,
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
        let (markdown, frontmatter) = read_source::<PostFrontmatter>(&source, parser)?;
        let PostFrontmatter {
            published,
            mut repost,
        } = frontmatter;
        if let Some(repost) = repost.as_mut() {
            repost.author = repost.author.trim().to_owned();
            if repost.author.is_empty() {
                bail!("`repost.author` in {} cannot be empty", source.display());
            }
            for value in [&mut repost.title, &mut repost.url] {
                *value = value
                    .take()
                    .map(|value| value.trim().to_owned())
                    .filter(|value| !value.is_empty());
            }
        }
        let document = document(
            root,
            &source,
            git,
            name.slug(),
            name.draft(),
            published,
            markdown,
        )?;
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
                repost,
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
        let (markdown, frontmatter) = read_source::<PageFrontmatter>(&source, parser)?;
        let document = document(
            root,
            &source,
            git,
            name.slug(),
            name.draft(),
            frontmatter.published,
            markdown,
        )?;
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

fn read_source<T>(source_path: &Path, parser: &Parser) -> Result<(String, T)>
where
    T: Default + DeserializeOwned,
{
    let markdown = fs::read_to_string(source_path)?;
    let frontmatter = match parser.frontmatter(&markdown).as_deref() {
        Some("") | None => T::default(),
        Some(frontmatter) => yaml_serde::from_str(frontmatter)?,
    };

    Ok((markdown, frontmatter))
}

fn document(
    root: &Path,
    source_path: &Path,
    git: Option<&GitIndex>,
    fallback_title: &str,
    draft: bool,
    published: Option<OffsetDateTime>,
    markdown: String,
) -> Result<Document> {
    let (inferred_published, updated) = infer_dates(source_path, git)?;
    let published = published.unwrap_or(inferred_published);
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

fn infer_dates(path: &Path, git: Option<&GitIndex>) -> Result<(OffsetDateTime, OffsetDateTime)> {
    let history = match git {
        Some(git) => git.file_info(path)?,
        None => None,
    };

    match history {
        Some(history) if history.dirty => Ok((
            history.created_at,
            timestamp(fs::metadata(path)?.modified()?),
        )),
        Some(history) => Ok((history.created_at, history.updated_at)),
        None => filesystem_dates(path),
    }
}

fn filesystem_dates(path: &Path) -> Result<(OffsetDateTime, OffsetDateTime)> {
    let metadata = fs::metadata(path)?;
    let updated = metadata.modified()?;
    let created = match metadata.created() {
        Ok(created) => created,
        Err(_) => updated,
    };

    Ok((timestamp(created), timestamp(updated)))
}

fn timestamp(value: SystemTime) -> OffsetDateTime {
    OffsetDateTime::from(value).replace_nanosecond(0).unwrap()
}
