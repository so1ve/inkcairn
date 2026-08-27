use std::path::PathBuf;

use comrak::options::Plugins;
use comrak::{Arena, format_html_with_plugins};
use time::Date;

use crate::content::{Document, Page, PagePath, Post, PostPath};
use crate::parser::Parser;

mod headings;
mod highlight;
mod stylesheet;

use headings::Headings;
use highlight::Highlighter;
pub use stylesheet::{render as stylesheet, search as search_stylesheet};

pub struct OutlineEntry {
    pub level: u8,
    pub id: String,
    pub title: String,
}

pub struct RenderedTitle {
    pub text: String,
    pub html: String,
}

pub struct RenderedDescription {
    pub text: String,
    pub html: String,
}

pub struct RenderedSection {
    pub id: Option<String>,
    pub titles: Vec<String>,
    pub text: String,
}

pub struct RenderedArticle {
    pub source: PathBuf,
    pub draft: bool,
    pub published: Date,
    pub updated: Date,
    pub title: RenderedTitle,
    pub html: String,
    pub outline: Vec<OutlineEntry>,
    pub sections: Vec<RenderedSection>,
}

pub struct RenderedPost {
    pub article: RenderedArticle,
    pub path: PostPath,
    pub description: Option<RenderedDescription>,
}

pub struct RenderedPage {
    pub article: RenderedArticle,
    pub path: PagePath,
}

pub struct Renderer {
    parser: Parser,
    highlighter: Highlighter,
}

impl Renderer {
    pub fn new(parser: Parser) -> Self {
        Self {
            parser,
            highlighter: Highlighter::default(),
        }
    }

    pub fn post(&self, source: Post) -> RenderedPost {
        let (article, description) = self.document(source.document, true);

        RenderedPost {
            article,
            path: source.path,
            description,
        }
    }

    pub fn page(&self, source: Page) -> RenderedPage {
        let (article, _) = self.document(source.document, false);

        RenderedPage {
            article,
            path: source.path,
        }
    }

    fn document(
        &self,
        source: Document,
        extract_description: bool,
    ) -> (RenderedArticle, Option<RenderedDescription>) {
        let arena = Arena::new();
        let root = self.parser.parse(&arena, &source.markdown);
        let title_heading = Parser::title_heading(root);
        let title = match title_heading {
            Some(heading) => {
                let text = Parser::plain_text([heading]);
                let mut html = String::new();
                for node in heading.children() {
                    format_html_with_plugins(
                        node,
                        &self.parser.options,
                        &mut html,
                        &Plugins::default(),
                    )
                    .unwrap();
                }

                RenderedTitle {
                    text: if text.is_empty() {
                        source.fallback_title.clone()
                    } else {
                        text
                    },
                    html: if html.trim().is_empty() {
                        escape_html(&source.fallback_title)
                    } else {
                        html
                    },
                }
            }
            None => RenderedTitle {
                text: source.fallback_title.clone(),
                html: escape_html(&source.fallback_title),
            },
        };
        let description = if extract_description {
            title_heading.and_then(|heading| {
                let nodes = Parser::description_nodes(heading);
                let text = Parser::plain_text(nodes.iter().copied());
                let mut html = String::new();
                let mut plugins = Plugins::default();
                plugins.render.codefence_syntax_highlighter = Some(&self.highlighter);
                for node in nodes {
                    format_html_with_plugins(node, &self.parser.options, &mut html, &plugins)
                        .unwrap();
                    node.detach();
                }

                (!html.trim().is_empty()).then_some(RenderedDescription { text, html })
            })
        } else {
            None
        };
        if let Some(heading) = title_heading {
            heading.detach();
        }
        let headings = Headings::new(root);
        let mut html = String::new();
        {
            let mut plugins = Plugins::default();
            plugins.render.codefence_syntax_highlighter = Some(&self.highlighter);
            plugins.render.heading_adapter = Some(&headings);
            format_html_with_plugins(root, &self.parser.options, &mut html, &plugins).unwrap();
        }
        let (outline, sections) = headings.into_parts();

        (
            RenderedArticle {
                source: source.source,
                draft: source.draft,
                published: source.published,
                updated: source.updated,
                title,
                html,
                outline,
                sections,
            },
            description,
        )
    }

    pub fn markdown(&self, source: &str) -> String {
        let arena = Arena::new();
        let root = self.parser.parse(&arena, source);
        let mut plugins = Plugins::default();
        plugins.render.codefence_syntax_highlighter = Some(&self.highlighter);
        let mut html = String::new();
        format_html_with_plugins(root, &self.parser.options, &mut html, &plugins).unwrap();

        html
    }
}

fn escape_html(value: &str) -> String {
    let mut html = String::new();
    for character in value.chars() {
        match character {
            '&' => html.push_str("&amp;"),
            '<' => html.push_str("&lt;"),
            '>' => html.push_str("&gt;"),
            '"' => html.push_str("&quot;"),
            '\'' => html.push_str("&#39;"),
            _ => html.push(character),
        }
    }

    html
}
