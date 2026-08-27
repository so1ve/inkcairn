use askama::Template;
use time::format_description::well_known::Rfc2822;
use time::{Date, Time};

use super::Site;
use crate::render::RenderedPost;

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
    published: String,
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
        let posts = posts
            .iter()
            .map(|post| FeedItem {
                title: &post.article.title.text,
                href: format!("{site_url}/{}", post.path.url),
                description: post
                    .description
                    .as_ref()
                    .map(|description| description.text.as_str()),
                published: rss_date(post.article.published),
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

fn rss_date(value: Date) -> String {
    value
        .with_time(Time::MIDNIGHT)
        .assume_utc()
        .format(&Rfc2822)
        .unwrap()
}
