use std::cmp::Ordering;
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use std::sync::LazyLock;

use anyhow::{Context, Result, bail};
use regex_lite::Regex;
use time::macros::format_description;
use time::{Date, OffsetDateTime};

static PINNED_POST: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^0([0-9])-(.*)$").unwrap());
static DATED_POST: LazyLock<Regex> =
    LazyLock::new(|| Regex::new(r"^([0-9]{4}-[0-9]{2}-[0-9]{2})-(?:([0-9]{2})-)?(.*)$").unwrap());
static ORDERED_PAGE: LazyLock<Regex> = LazyLock::new(|| Regex::new(r"^[0-9]{2}-(.+)$").unwrap());

#[derive(Clone, Copy)]
enum PostPosition {
    Pinned(u8),
    Dated(Option<u8>),
}

pub struct PostName<'a> {
    slug: &'a str,
    draft: bool,
    date: Option<Date>,
    position: PostPosition,
}

pub struct PageName<'a> {
    slug: &'a str,
    draft: bool,
}

struct Ordered<T> {
    value: T,
    source: PathBuf,
    published: OffsetDateTime,
    position: PostPosition,
}

pub struct Posts<T> {
    entries: Vec<Ordered<T>>,
    pinned_positions: HashMap<u8, PathBuf>,
    dated_positions: HashMap<(Date, u8), PathBuf>,
}

impl PostName<'_> {
    pub const fn slug(&self) -> &str {
        self.slug
    }

    pub const fn draft(&self) -> bool {
        self.draft
    }

    pub const fn pinned(&self) -> bool {
        matches!(self.position, PostPosition::Pinned(_))
    }
}

impl PageName<'_> {
    pub const fn slug(&self) -> &str {
        self.slug
    }

    pub const fn draft(&self) -> bool {
        self.draft
    }
}

impl<T> Posts<T> {
    pub fn new() -> Self {
        Self {
            entries: Vec::new(),
            pinned_positions: HashMap::new(),
            dated_positions: HashMap::new(),
        }
    }

    pub fn push(
        &mut self,
        source: &Path,
        published: OffsetDateTime,
        name: &PostName<'_>,
        value: T,
    ) -> Result<()> {
        if let Some(date) = name.date
            && date != published.date()
        {
            bail!(
                "{} has date prefix `{date}` but publication date `{}`",
                published.date(),
                source.display()
            );
        }

        match name.position {
            PostPosition::Pinned(position) => {
                if let Some(existing) = self.pinned_positions.insert(position, source.to_owned()) {
                    bail!(
                        "{} and {} use the same pinned position `{position:02}-`",
                        existing.display(),
                        source.display()
                    );
                }
            }
            PostPosition::Dated(Some(position)) => {
                if let Some(existing) = self
                    .dated_positions
                    .insert((published.date(), position), source.to_owned())
                {
                    bail!(
                        "{} and {} use the same dated position `{}-{position:02}-`",
                        existing.display(),
                        source.display(),
                        published.date()
                    );
                }
            }
            PostPosition::Dated(None) => {}
        }

        self.entries.push(Ordered {
            value,
            source: source.to_owned(),
            published,
            position: name.position,
        });

        Ok(())
    }

    pub fn into_values(mut self) -> Vec<T> {
        self.entries.sort_by(compare_posts);
        self.entries.into_iter().map(|entry| entry.value).collect()
    }
}

pub fn post_name(path: &Path) -> Result<PostName<'_>> {
    let (stem, draft) = document_stem(path);
    if let Some(captures) = PINNED_POST.captures(stem) {
        let position = captures.get(1).unwrap().as_str().parse().unwrap();
        let slug = captures.get(2).unwrap().as_str();
        if slug.is_empty() {
            bail!("pinned post has no slug");
        }

        return Ok(PostName {
            slug,
            draft,
            date: None,
            position: PostPosition::Pinned(position),
        });
    }

    if let Some(captures) = DATED_POST.captures(stem) {
        let date_source = captures.get(1).unwrap().as_str();
        let date = Date::parse(date_source, &format_description!("[year]-[month]-[day]"))
            .with_context(|| format!("invalid date prefix `{date_source}`"))?;
        let position = captures
            .get(2)
            .map(|position| position.as_str().parse().unwrap());
        let slug = captures.get(3).unwrap().as_str();
        if slug.is_empty() {
            bail!("dated post has no slug");
        }

        return Ok(PostName {
            slug,
            draft,
            date: Some(date),
            position: PostPosition::Dated(position),
        });
    }

    Ok(PostName {
        slug: stem,
        draft,
        date: None,
        position: PostPosition::Dated(None),
    })
}

pub fn page_name(path: &Path) -> PageName<'_> {
    let (stem, draft) = document_stem(path);
    let slug = if let Some(captures) = ORDERED_PAGE.captures(stem) {
        captures.get(1).unwrap().as_str()
    } else {
        stem
    };

    PageName { slug, draft }
}

fn document_stem(path: &Path) -> (&str, bool) {
    let stem = path.file_stem().unwrap().to_str().unwrap();
    match stem.strip_suffix(".draft") {
        Some(stem) => (stem, true),
        None => (stem, false),
    }
}

fn compare_posts<T>(left: &Ordered<T>, right: &Ordered<T>) -> Ordering {
    match (left.position, right.position) {
        (PostPosition::Pinned(left_position), PostPosition::Pinned(right_position)) => {
            left_position
                .cmp(&right_position)
                .then_with(|| left.source.cmp(&right.source))
        }
        (PostPosition::Pinned(_), PostPosition::Dated(_)) => Ordering::Less,
        (PostPosition::Dated(_), PostPosition::Pinned(_)) => Ordering::Greater,
        (PostPosition::Dated(left_position), PostPosition::Dated(right_position)) => right
            .published
            .date()
            .cmp(&left.published.date())
            .then_with(|| {
                left_position
                    .unwrap_or(u8::MAX)
                    .cmp(&right_position.unwrap_or(u8::MAX))
            })
            .then_with(|| right.published.cmp(&left.published))
            .then_with(|| left.source.cmp(&right.source)),
    }
}
