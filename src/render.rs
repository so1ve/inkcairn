use std::path::PathBuf;
use std::sync::Mutex;

use anyhow::{Context, Result};
use comrak::html::escape;
use comrak::options::Plugins;
use comrak::{Arena, Node, format_html_with_plugins};
use time::Date;

use crate::content::{Document, Page, PagePath, Post, PostPath, Repost};
use crate::parser::Parser;

mod devices;
mod friends;
mod headings;
mod highlight;

use headings::Headings;
use highlight::Highlighter;

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
    pub pinned: bool,
    pub repost: Option<Repost>,
}

pub struct RenderedPage {
    pub article: RenderedArticle,
    pub path: PagePath,
}

pub struct Renderer {
    parser: Parser,
    highlighter: Highlighter,
}

const FRIENDS_LANGUAGE: &str = "friends";
const DEVICES_LANGUAGE: &str = "devices";

#[derive(Default)]
struct FriendLinks {
    error: Mutex<Option<anyhow::Error>>,
}

#[derive(Default)]
struct DeviceCards {
    error: Mutex<Option<anyhow::Error>>,
}

impl FriendLinks {
    fn take_error(&self) -> Option<anyhow::Error> {
        self.error.lock().unwrap().take()
    }
}

impl DeviceCards {
    fn take_error(&self) -> Option<anyhow::Error> {
        self.error.lock().unwrap().take()
    }
}

impl Renderer {
    pub fn new(parser: Parser) -> Self {
        Self {
            parser,
            highlighter: Highlighter::default(),
        }
    }

    pub fn post(&self, source: Post) -> Result<RenderedPost> {
        let (article, description) = self.document(source.document, true)?;

        Ok(RenderedPost {
            article,
            path: source.path,
            description,
            pinned: source.pinned,
            repost: source.repost,
        })
    }

    pub fn page(&self, source: Page) -> Result<RenderedPage> {
        let (article, _) = self.document(source.document, false)?;

        Ok(RenderedPage {
            article,
            path: source.path,
        })
    }

    fn document(
        &self,
        source: Document,
        extract_description: bool,
    ) -> Result<(RenderedArticle, Option<RenderedDescription>)> {
        let arena = Arena::new();
        let root = self.parser.parse(&arena, &source.markdown);
        let title_heading = Parser::title_heading(root);
        let mut fallback_title_html = String::new();
        // fallback title is plaintext. escape if it contains any HTML
        escape(&mut fallback_title_html, &source.fallback_title).unwrap();
        let title = match title_heading {
            Some(heading) => {
                let text = Parser::plain_text([heading]);
                let html = self
                    .html(heading.children(), None)
                    .with_context(|| format!("failed to render {}", source.source.display()))?;

                RenderedTitle {
                    text: if text.is_empty() {
                        source.fallback_title.clone()
                    } else {
                        text
                    },
                    html: if html.trim().is_empty() {
                        fallback_title_html
                    } else {
                        html
                    },
                }
            }
            None => RenderedTitle {
                text: source.fallback_title.clone(),
                html: fallback_title_html,
            },
        };
        let description = if extract_description {
            if let Some(heading) = title_heading {
                let nodes = Parser::description_nodes(heading);
                let text = Parser::plain_text(nodes.iter().copied());
                let html = self
                    .html(nodes.iter().copied(), None)
                    .with_context(|| format!("failed to render {}", source.source.display()))?;
                for node in nodes {
                    node.detach();
                }

                (!html.trim().is_empty()).then_some(RenderedDescription { text, html })
            } else {
                None
            }
        } else {
            None
        };
        if let Some(heading) = title_heading {
            heading.detach();
        }
        let headings = Headings::new(root);
        let html = self
            .html([root], Some(&headings))
            .with_context(|| format!("failed to render {}", source.source.display()))?;
        let (outline, sections) = headings.into_parts();

        Ok((
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
        ))
    }

    pub fn markdown(&self, source: &str) -> Result<String> {
        let arena = Arena::new();
        let root = self.parser.parse(&arena, source);

        self.html([root], None)
    }

    fn html<'a>(
        &self,
        nodes: impl IntoIterator<Item = Node<'a>>,
        headings: Option<&Headings>,
    ) -> Result<String> {
        let friends = FriendLinks::default();
        let devices = DeviceCards::default();
        let mut plugins = Plugins::default();
        plugins
            .render
            .codefence_renderers
            .insert(FRIENDS_LANGUAGE.to_owned(), &friends);
        plugins
            .render
            .codefence_renderers
            .insert(DEVICES_LANGUAGE.to_owned(), &devices);
        plugins.render.codefence_syntax_highlighter = Some(&self.highlighter);
        if let Some(headings) = headings {
            plugins.render.heading_adapter = Some(headings);
        }
        let mut html = String::new();
        for node in nodes {
            if let Err(error) =
                format_html_with_plugins(node, &self.parser.options, &mut html, &plugins)
            {
                return Err(friends
                    .take_error()
                    .or_else(|| devices.take_error())
                    .unwrap_or_else(|| error.into()));
            }
        }

        Ok(html)
    }
}
