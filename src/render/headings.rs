use std::collections::{HashMap, VecDeque};
use std::fmt;
use std::sync::Mutex;

use comrak::adapters::{HeadingAdapter, HeadingMeta};
use comrak::nodes::{NodeValue, Sourcepos};

use super::{OutlineEntry, RenderedSection};
use crate::parser::Parser;

pub struct Headings {
    ids: Mutex<VecDeque<String>>,
    outline: Vec<OutlineEntry>,
    sections: Vec<RenderedSection>,
}

impl Headings {
    pub fn new(root: comrak::Node<'_>) -> Self {
        let mut counts = HashMap::<String, usize>::new();
        let mut ids = VecDeque::new();
        let mut outline = Vec::new();
        let mut sections = vec![RenderedSection {
            id: None,
            titles: Vec::new(),
            text: String::new(),
        }];
        let mut content = Vec::new();
        let mut titles = Vec::<(u8, String)>::new();

        for node in root.children() {
            let NodeValue::Heading(heading) = &node.data().value else {
                content.push(node);

                continue;
            };
            let level = heading.level;

            sections.last_mut().unwrap().text = Parser::plain_text(content.drain(..));

            let title = Parser::plain_text([node]);
            let base = heading_id(&title);
            let occurrence = counts.entry(base.clone()).or_default();
            *occurrence += 1;
            let id = if *occurrence == 1 {
                base
            } else {
                format!("{base}-{occurrence}")
            };
            ids.push_back(id.clone());

            while titles
                .last()
                .is_some_and(|(parent_level, _)| *parent_level >= level)
            {
                titles.pop();
            }
            titles.push((level, title.clone()));

            if matches!(level, 2 | 3) {
                outline.push(OutlineEntry {
                    level,
                    id: id.clone(),
                    title,
                });
            }
            sections.push(RenderedSection {
                id: Some(id),
                titles: titles.iter().map(|(_, title)| title.clone()).collect(),
                text: String::new(),
            });
        }
        sections.last_mut().unwrap().text = Parser::plain_text(content);

        Self {
            ids: Mutex::new(ids),
            outline,
            sections,
        }
    }

    pub fn into_parts(self) -> (Vec<OutlineEntry>, Vec<RenderedSection>) {
        assert!(self.ids.into_inner().unwrap().is_empty());

        (self.outline, self.sections)
    }
}

impl HeadingAdapter for Headings {
    fn enter(
        &self,
        output: &mut dyn fmt::Write,
        heading: &HeadingMeta,
        _source_position: Option<Sourcepos>,
    ) -> fmt::Result {
        let id = self.ids.lock().unwrap().pop_front().unwrap();

        write!(
            output,
            "<h{level} id=\"{id}\"><a class=\"anchor\" href=\"#{id}\" aria-hidden=\"true\">#</a>",
            level = heading.level
        )
    }

    fn exit(&self, output: &mut dyn fmt::Write, heading: &HeadingMeta) -> fmt::Result {
        writeln!(output, "</h{}>", heading.level)
    }
}

fn heading_id(title: &str) -> String {
    let mut id = String::new();
    let mut separator = false;
    for character in title.trim().chars().flat_map(char::to_lowercase) {
        if character.is_alphanumeric() {
            if separator && !id.is_empty() {
                id.push('-');
            }
            id.push(character);
            separator = false;
        } else {
            separator = true;
        }
    }
    if id.is_empty() {
        "section".to_owned()
    } else {
        id
    }
}
