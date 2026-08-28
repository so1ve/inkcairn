mod article;
mod category;
mod feed;
mod layout;
mod listing;
mod search;

use std::cmp::Reverse;

use crate::comments::CommentSnapshot;
use crate::content::CategoryPath;
use crate::metadata::Metadata;
use crate::render::{RenderedPage, RenderedPost};
use crate::url_path;

pub struct Build {
    pub label: String,
    pub url: Option<String>,
}

pub struct Snippets<'a> {
    pub head: Option<&'a str>,
    pub home: Option<&'a str>,
    pub after_content: Option<&'a str>,
}

pub struct Site<'a> {
    metadata: &'a Metadata,
    build: Option<Build>,
    pages: &'a [RenderedPage],
    comments: &'a CommentSnapshot,
    snippets: Snippets<'a>,
    root_url: String,
    generator: &'static str,
}

pub struct PageContext<'a, 'site> {
    site: &'a Site<'site>,
    title: String,
    canonical_url: Option<String>,
    noindex: bool,
    navigation: Vec<Navigation<'a>>,
}

struct Navigation<'a> {
    href: String,
    label: &'a str,
    current: bool,
}

fn chronological_posts(posts: &[RenderedPost]) -> Vec<&RenderedPost> {
    let mut posts = posts.iter().collect::<Vec<_>>();
    posts.sort_by_key(|post| Reverse(post.article.published));
    posts
}

pub struct CategoryLink<'a> {
    pub href: String,
    pub label: &'a str,
}

impl<'a> Site<'a> {
    pub fn new(
        metadata: &'a Metadata,
        build: Option<Build>,
        pages: &'a [RenderedPage],
        comments: &'a CommentSnapshot,
        snippets: Snippets<'a>,
    ) -> Self {
        let root_url = url_path::base(metadata.url.as_deref());

        Self {
            metadata,
            build,
            pages,
            comments,
            snippets,
            root_url,
            generator: crate::GENERATOR,
        }
    }

    fn href(&self, path: &str) -> String {
        format!("{}/{path}", self.root_url)
    }

    fn category_links<'path>(&self, path: &'path [CategoryPath]) -> Vec<CategoryLink<'path>> {
        path.iter()
            .map(|category| CategoryLink {
                href: self.href(&category.url),
                label: &category.label,
            })
            .collect()
    }
}
