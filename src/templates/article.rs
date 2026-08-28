use askama::Template;

use super::{CategoryLink, PageContext, Site};
use crate::content::Repost;
use crate::render::{RenderedArticle, RenderedPage, RenderedPost};

#[derive(Template)]
#[template(path = "article.html")]
struct ArticleTemplate<'a, 'site> {
    page: PageContext<'a, 'site>,
    article: &'a RenderedArticle,
    categories: Vec<CategoryLink<'a>>,
    after_content: Option<&'site str>,
    pinned: bool,
    repost: Option<&'a Repost>,
}

impl<'site> Site<'site> {
    pub fn post_article<'a>(&'a self, post: &'a RenderedPost) -> String {
        self.article(
            &post.path.url,
            &post.article,
            self.category_links(&post.path.categories),
            post.pinned,
            post.repost.as_ref(),
        )
    }

    pub fn page_article<'a>(&'a self, page: &'a RenderedPage) -> String {
        self.article(&page.path.url, &page.article, Vec::new(), false, None)
    }

    fn article<'a>(
        &'a self,
        path: &str,
        article: &'a RenderedArticle,
        categories: Vec<CategoryLink<'a>>,
        pinned: bool,
        repost: Option<&'a Repost>,
    ) -> String {
        let mut page = self.page(path, &article.title.text);
        if let Some(repost) = repost {
            page.noindex = true;
            if let Some(url) = repost.url.as_deref() {
                page.canonical_url = Some(url.to_owned());
            }
        }

        ArticleTemplate {
            page,
            article,
            categories,
            after_content: self.snippets.after_content,
            pinned,
            repost,
        }
        .render()
        .unwrap()
    }
}
