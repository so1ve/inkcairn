use std::path::PathBuf;

use serde::Serialize;
use time::Date;

use crate::output::OutputFile;
use crate::render::{RenderedPage, RenderedPost};

#[derive(Serialize)]
struct SearchDocument<'a> {
    id: usize,
    url: String,
    title: &'a str,
    breadcrumbs: String,
    published: Option<Date>,
    categories: String,
    content: &'a str,
}

pub fn documents(posts: &[RenderedPost], pages: &[RenderedPage]) -> OutputFile {
    let mut documents = Vec::new();

    for post in posts {
        let categories = post
            .path
            .categories
            .iter()
            .map(|category| category.label.as_str())
            .collect::<Vec<_>>()
            .join(" / ");

        for section in &post.article.sections {
            let (title, breadcrumbs) = match section.titles.split_last() {
                Some((title, parents)) => (
                    title.as_str(),
                    std::iter::once(post.article.title.text.as_str())
                        .chain(parents.iter().map(String::as_str))
                        .collect::<Vec<_>>()
                        .join(" / "),
                ),
                None => (post.article.title.text.as_str(), String::new()),
            };
            let content = match (&section.id, &post.description) {
                (None, Some(description)) => description.text.as_str(),
                _ => section.text.as_str(),
            };
            let url = match &section.id {
                Some(id) => format!("{}#{id}", post.path.url),
                None => post.path.url.clone(),
            };
            documents.push(SearchDocument {
                id: documents.len(),
                url,
                title,
                breadcrumbs,
                published: Some(post.article.published),
                categories: categories.clone(),
                content,
            });
        }
    }

    for page in pages {
        for section in &page.article.sections {
            let (title, breadcrumbs) = match section.titles.split_last() {
                Some((title, parents)) => (
                    title.as_str(),
                    std::iter::once(page.article.title.text.as_str())
                        .chain(parents.iter().map(String::as_str))
                        .collect::<Vec<_>>()
                        .join(" / "),
                ),
                None => (page.article.title.text.as_str(), String::new()),
            };
            let url = match &section.id {
                Some(id) => format!("{}#{id}", page.path.url),
                None => page.path.url.clone(),
            };
            documents.push(SearchDocument {
                id: documents.len(),
                url,
                title,
                breadcrumbs,
                published: None,
                categories: String::new(),
                content: &section.text,
            });
        }
    }

    OutputFile {
        path: PathBuf::from("search-index.json"),
        bytes: serde_json::to_vec(&documents).unwrap(),
        source: None,
    }
}
