use askama::Template;

use super::listing::PostLink;
use super::{CategoryLink, PageContext, Site};
use crate::categories::Category;
use crate::render::RenderedPost;

#[derive(Template)]
#[template(path = "category.html")]
struct CategoryTemplate<'a, 'site> {
    page: PageContext<'a, 'site>,
    ancestors: Vec<CategoryLink<'a>>,
    current: &'a str,
    categories: Vec<CategoryLink<'a>>,
    posts: Vec<PostLink<'a>>,
}

impl<'site> Site<'site> {
    pub fn category<'a>(&'a self, category: &'a Category, all_posts: &'a [RenderedPost]) -> String {
        let current = category.path.last().unwrap();
        let title = category
            .path
            .iter()
            .map(|category| category.label.as_str())
            .collect::<Vec<_>>()
            .join(" / ");
        let ancestors = self.category_links(&category.path[..category.path.len() - 1]);
        let categories = category
            .children
            .iter()
            .map(|child| {
                let current = child.path.last().unwrap();

                CategoryLink {
                    href: self.href(&current.url),
                    label: &current.label,
                }
            })
            .collect();
        let posts = category.posts.iter().map(|index| &all_posts[*index]);

        CategoryTemplate {
            page: self.page(&current.url, &title),
            ancestors,
            current: &current.label,
            categories,
            posts: self.post_links(posts, category.path.len()),
        }
        .render()
        .unwrap()
    }
}
