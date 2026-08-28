use askama::Template;
use time::format_description::well_known::Rfc2822;

use super::{Site, chronological_posts};
use crate::content::Repost;
use crate::render::RenderedPost;

const FEED_POST_LIMIT: usize = 20;

#[derive(Template)]
#[template(path = "rss.xml")]
struct FeedTemplate<'a> {
    site: FeedChannel<'a>,
    posts: &'a [FeedItem<'a>],
}

struct FeedChannel<'a> {
    title: &'a str,
    href: &'a str,
    description: &'a str,
}

struct FeedItem<'a> {
    title: &'a str,
    href: String,
    description: Option<&'a str>,
    repost: Option<&'a Repost>,
    repost_published: Option<String>,
    content: String,
    published: String,
}

#[derive(Template)]
#[template(path = "repost-notice.html")]
struct RepostNoticeTemplate<'a> {
    repost: &'a Repost,
    repost_published: Option<String>,
    repost_published_label: Option<String>,
}

#[derive(Template)]
#[template(path = "sitemap.xml")]
struct SitemapTemplate<'a> {
    urls: &'a [String],
}

impl Site<'_> {
    pub fn feed(&self, posts: &[RenderedPost]) -> String {
        let site_url = self.metadata.url.as_deref().unwrap();
        let description = match self.metadata.description.as_deref() {
            Some(description) => description,
            None => &self.metadata.title,
        };
        let href = format!("{site_url}/");
        let posts = chronological_posts(posts)
            .into_iter()
            .take(FEED_POST_LIMIT)
            .map(|post| {
                let published = post.article.published.format(&Rfc2822).unwrap();

                let mut content = String::with_capacity(post.article.html.len());
                if let Some(repost) = post.repost.as_ref() {
                    content.push_str(
                        &RepostNoticeTemplate {
                            repost,
                            repost_published: repost.published.map(crate::date_time::rfc3339),
                            repost_published_label: repost.published.map(crate::date_time::date),
                        }
                        .render()
                        .unwrap(),
                    );
                }
                content.push_str(&post.article.html);

                FeedItem {
                    title: &post.article.title.text,
                    href: format!("{site_url}/{}", post.path.url),
                    description: post
                        .description
                        .as_ref()
                        .map(|description| description.text.as_str()),
                    repost: post.repost.as_ref(),
                    repost_published: post
                        .repost
                        .as_ref()
                        .and_then(|repost| repost.published)
                        .map(crate::date_time::date),
                    content,
                    published,
                }
            })
            .collect::<Vec<_>>();

        FeedTemplate {
            site: FeedChannel {
                title: &self.metadata.title,
                href: &href,
                description,
            },
            posts: &posts,
        }
        .render()
        .unwrap()
    }

    pub fn sitemap(&self, paths: &[String]) -> String {
        let site_url = self.metadata.url.as_deref().unwrap();
        let urls = paths
            .iter()
            .map(|path| format!("{site_url}/{path}"))
            .collect::<Vec<_>>();

        SitemapTemplate { urls: &urls }.render().unwrap()
    }
}
