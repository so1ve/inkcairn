use std::fs;
use std::path::Path;

use anyhow::{Context, Result, bail};
use comrak::Arena;
use serde::Deserialize;

use crate::parser::Parser;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Frontmatter {
    #[serde(default = "default_language")]
    language: String,
    url: Option<String>,
}

impl Default for Frontmatter {
    fn default() -> Self {
        Self {
            language: default_language(),
            url: None,
        }
    }
}

pub struct Metadata {
    pub language: String,
    pub title: String,
    pub description: Option<String>,
    pub url: Option<String>,
}

fn default_language() -> String {
    "en".to_owned()
}

impl Metadata {
    pub fn load(root: &Path, parser: &Parser) -> Result<Self> {
        let path = root.join("inkcairn.md");
        let markdown = fs::read_to_string(&path)
            .with_context(|| format!("could not read {}", path.display()))?;
        let frontmatter = match parser.frontmatter(&markdown).as_deref() {
            Some("") | None => Frontmatter::default(),
            Some(frontmatter) => yaml_serde::from_str(frontmatter)?,
        };
        let language = frontmatter.language.trim().to_owned();
        if language.is_empty() {
            bail!("`language` cannot be empty");
        }
        let url = frontmatter
            .url
            .map(|url| url.trim().trim_end_matches('/').to_owned());

        let arena = Arena::new();
        let document = parser.parse(&arena, &markdown);
        let Some(title_heading) = Parser::title_heading(document) else {
            bail!("{} must contain a level-one heading", path.display());
        };
        let title = Parser::plain_text([title_heading]);
        if title.is_empty() {
            bail!(
                "the level-one heading in {} cannot be empty",
                path.display()
            );
        }
        let description = Parser::plain_text(Parser::description_nodes(title_heading));
        let description = (!description.is_empty()).then_some(description);

        Ok(Self {
            language,
            title,
            description,
            url,
        })
    }
}
