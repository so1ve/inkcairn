use askama::Template;

use super::{CategoryLink, PageContext, Site};
use crate::render::{RenderedArticle, RenderedPage, RenderedPost};

#[derive(Template)]
#[template(path = "article.html")]
struct ArticleTemplate<'a, 'site> {
    page: PageContext<'a, 'site>,
    article: &'a RenderedArticle,
    categories: Vec<CategoryLink<'a>>,
    after_content: Option<&'site str>,
    pinned: bool,
}

impl<'site> Site<'site> {
    pub fn post_article<'a>(&'a self, post: &'a RenderedPost) -> String {
        self.article(
            &post.path.url,
            &post.article,
            self.category_links(&post.path.categories),
            post.pinned,
        )
    }

    pub fn page_article<'a>(&'a self, page: &'a RenderedPage) -> String {
        self.article(&page.path.url, &page.article, Vec::new(), false)
    }

    fn article<'a>(
        &'a self,
        path: &str,
        article: &'a RenderedArticle,
        categories: Vec<CategoryLink<'a>>,
        pinned: bool,
    ) -> String {
        ArticleTemplate {
            page: self.page(path, &article.title.text),
            article,
            categories,
            after_content: self.snippets.after_content,
            pinned,
        }
        .render()
        .unwrap()
    }
}
