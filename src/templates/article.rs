use askama::Template;

use super::{CategoryLink, PageContext, Site};
use crate::comments::Discussion;
use crate::content::{CategoryPath, Repost};
use crate::metadata::Giscus;
use crate::render::{RenderedArticle, RenderedDescription, RenderedPage, RenderedPost};

#[derive(Template)]
#[template(path = "article.html")]
struct ArticleTemplate<'a, 'site> {
    page: PageContext<'a, 'site>,
    article: &'a RenderedArticle,
    description: Option<&'a str>,
    published: String,
    published_label: String,
    updated: String,
    updated_label: String,
    categories: Vec<CategoryLink<'a>>,
    after_content: Option<&'site str>,
    pinned: bool,
    repost: Option<&'a Repost>,
    repost_published: Option<String>,
    repost_published_label: Option<String>,
    comments: Option<CommentSection<'a>>,
}

struct CommentSection<'a> {
    giscus: &'a Giscus,
    term: &'a str,
    snapshot_available: bool,
    discussion: Option<&'a Discussion>,
}

#[derive(Clone, Copy)]
struct ArticleContext<'a> {
    article: &'a RenderedArticle,
    path: &'a str,
    description: Option<&'a RenderedDescription>,
    categories: &'a [CategoryPath],
    pinned: bool,
    repost: Option<&'a Repost>,
}

impl<'site> Site<'site> {
    pub fn post_article<'a>(&'a self, post: &'a RenderedPost) -> String {
        self.article(ArticleContext {
            article: &post.article,
            path: &post.path.url,
            description: post.description.as_ref(),
            categories: &post.path.categories,
            pinned: post.pinned,
            repost: post.repost.as_ref(),
        })
    }

    pub fn page_article<'a>(&'a self, page: &'a RenderedPage) -> String {
        self.article(ArticleContext {
            article: &page.article,
            path: &page.path.url,
            description: None,
            categories: &[],
            pinned: false,
            repost: None,
        })
    }

    fn comment_section<'a>(&'a self, term: &'a str) -> Option<CommentSection<'a>> {
        self.metadata
            .comments
            .as_ref()
            .map(|giscus| CommentSection {
                giscus,
                term,
                snapshot_available: self.comments.available(),
                discussion: self.comments.discussion(term),
            })
    }

    fn article<'a>(&'a self, context: ArticleContext<'a>) -> String {
        let ArticleContext {
            article,
            path,
            description,
            categories,
            pinned,
            repost,
        } = context;
        let mut page = self.page(path, &article.title.text);
        if let Some(description) = description {
            page.description = Some(&description.text);
        }
        if let Some(repost) = repost {
            page.noindex = true;
            if let Some(url) = repost.url.as_deref() {
                page.canonical_url = Some(url.to_owned());
            }
        }

        ArticleTemplate {
            page,
            article,
            description: description.map(|description| description.html.as_str()),
            published: crate::date_time::rfc3339(article.published),
            published_label: crate::date_time::display(article.published),
            updated: crate::date_time::rfc3339(article.updated),
            updated_label: crate::date_time::display(article.updated),
            categories: self.category_links(categories),
            after_content: self.snippets.after_content,
            pinned,
            repost,
            repost_published: repost
                .and_then(|repost| repost.published)
                .map(crate::date_time::rfc3339),
            repost_published_label: repost
                .and_then(|repost| repost.published)
                .map(crate::date_time::date),
            comments: self.comment_section(path),
        }
        .render()
        .unwrap()
    }
}
