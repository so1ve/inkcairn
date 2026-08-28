use askama::Template;

use super::{CategoryLink, PageContext, Site};
use crate::comments::Discussion;
use crate::content::Repost;
use crate::metadata::Giscus;
use crate::render::{RenderedArticle, RenderedPage, RenderedPost};

#[derive(Template)]
#[template(path = "article.html")]
struct ArticleTemplate<'a, 'site> {
    page: PageContext<'a, 'site>,
    article: &'a RenderedArticle,
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

impl<'site> Site<'site> {
    pub fn post_article<'a>(&'a self, post: &'a RenderedPost) -> String {
        let mut page = self.page(&post.path.url, &post.article.title.text);
        if let Some(description) = post.description.as_ref() {
            page.description = Some(&description.text);
        }

        self.article(
            page,
            &post.article,
            self.category_links(&post.path.categories),
            post.pinned,
            post.repost.as_ref(),
            self.comment_section(&post.path.url),
        )
    }

    pub fn page_article<'a>(&'a self, page: &'a RenderedPage) -> String {
        self.article(
            self.page(&page.path.url, &page.article.title.text),
            &page.article,
            Vec::new(),
            false,
            None,
            self.comment_section(&page.path.url),
        )
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

    fn article<'a>(
        &'a self,
        mut page: PageContext<'a, 'site>,
        article: &'a RenderedArticle,
        categories: Vec<CategoryLink<'a>>,
        pinned: bool,
        repost: Option<&'a Repost>,
        comments: Option<CommentSection<'a>>,
    ) -> String {
        if let Some(repost) = repost {
            page.noindex = true;
            if let Some(url) = repost.url.as_deref() {
                page.canonical_url = Some(url.to_owned());
            }
        }

        ArticleTemplate {
            page,
            article,
            published: crate::date_time::rfc3339(article.published),
            published_label: crate::date_time::display(article.published),
            updated: crate::date_time::rfc3339(article.updated),
            updated_label: crate::date_time::display(article.updated),
            categories,
            after_content: self.snippets.after_content,
            pinned,
            repost,
            repost_published: repost
                .and_then(|repost| repost.published)
                .map(crate::date_time::rfc3339),
            repost_published_label: repost
                .and_then(|repost| repost.published)
                .map(crate::date_time::date),
            comments,
        }
        .render()
        .unwrap()
    }
}
