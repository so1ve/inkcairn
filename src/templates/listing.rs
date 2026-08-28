use askama::Template;
use time::Date;

use super::{CategoryLink, PageContext, Site, chronological_posts};
use crate::categories::Category;
use crate::render::RenderedPost;

pub struct PostLink<'a> {
    pub href: String,
    pub title: &'a str,
    pub published: Date,
    pub description: Option<&'a str>,
    pub categories: Vec<CategoryLink<'a>>,
    pub draft: bool,
    pub pinned: bool,
    pub repost: bool,
}

struct ArchiveYear<'a> {
    year: i32,
    posts: Vec<PostLink<'a>>,
}

#[derive(Template)]
#[template(path = "home.html")]
struct HomeTemplate<'a, 'site> {
    page: PageContext<'a, 'site>,
    posts: Vec<PostLink<'a>>,
}

#[derive(Template)]
#[template(path = "archive.html")]
struct ArchiveTemplate<'a, 'site> {
    page: PageContext<'a, 'site>,
    years: Vec<ArchiveYear<'a>>,
}

#[derive(Template)]
#[template(path = "posts.html")]
struct PostsTemplate<'a, 'site> {
    page: PageContext<'a, 'site>,
    categories: Vec<CategoryLink<'a>>,
    posts: Vec<PostLink<'a>>,
}

impl<'site> Site<'site> {
    pub fn home<'a>(&'a self, posts: &'a [RenderedPost]) -> String {
        HomeTemplate {
            page: self.page("", &self.metadata.title),
            posts: self.post_links(posts.iter().take(3), 0),
        }
        .render()
        .unwrap()
    }

    pub fn archive<'a>(&'a self, posts: &'a [RenderedPost]) -> String {
        let mut years = Vec::<ArchiveYear<'a>>::new();
        let posts = chronological_posts(posts);
        for post in self.post_links(posts, 0) {
            let year = post.published.year();

            if let Some(group) = years.last_mut()
                && group.year == year
            {
                group.posts.push(post);
            } else {
                years.push(ArchiveYear {
                    year,
                    posts: vec![post],
                });
            }
        }

        ArchiveTemplate {
            page: self.page("archive.html", "Archive"),
            years,
        }
        .render()
        .unwrap()
    }

    pub fn posts<'a>(&'a self, categories: &'a [Category], posts: &'a [RenderedPost]) -> String {
        let categories = categories
            .iter()
            .map(|category| {
                let current = category.path.last().unwrap();

                CategoryLink {
                    href: self.href(&current.url),
                    label: &current.label,
                }
            })
            .collect();

        PostsTemplate {
            page: self.page("posts.html", "Posts"),
            categories,
            posts: self.post_links(posts, 0),
        }
        .render()
        .unwrap()
    }

    pub fn post_links<'a>(
        &self,
        posts: impl IntoIterator<Item = &'a RenderedPost>,
        category_depth: usize,
    ) -> Vec<PostLink<'a>> {
        posts
            .into_iter()
            .map(|post| PostLink {
                href: self.href(&post.path.url),
                title: &post.article.title.html,
                published: post.article.published,
                description: post
                    .description
                    .as_ref()
                    .map(|description| description.html.as_str()),
                categories: self.category_links(&post.path.categories[category_depth..]),
                draft: post.article.draft,
                pinned: post.pinned,
                repost: post.repost.is_some(),
            })
            .collect()
    }
}
