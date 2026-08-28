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
    comments: Option<Giscus>,
}

impl Default for Frontmatter {
    fn default() -> Self {
        Self {
            language: default_language(),
            url: None,
            comments: None,
        }
    }
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct Giscus {
    pub repo: String,
    pub repo_id: String,
    pub category: String,
    pub category_id: String,
}

pub struct Metadata {
    pub language: String,
    pub title: String,
    pub description: Option<String>,
    pub url: Option<String>,
    pub comments: Option<Giscus>,
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
        let comments = frontmatter.comments.map(Giscus::validate).transpose()?;

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
            comments,
        })
    }
}

impl Giscus {
    fn validate(mut self) -> Result<Self> {
        self.repo = self.repo.trim().to_owned();
        self.repo_id = self.repo_id.trim().to_owned();
        self.category = self.category.trim().to_owned();
        self.category_id = self.category_id.trim().to_owned();

        let Some((owner, name)) = self.repo.split_once('/') else {
            bail!("`comments.repo` must use the `owner/repository` format");
        };
        if owner.is_empty() || name.is_empty() || name.contains('/') {
            bail!("`comments.repo` must use the `owner/repository` format");
        }
        for (field, value) in [
            ("repo_id", self.repo_id.as_str()),
            ("category", self.category.as_str()),
            ("category_id", self.category_id.as_str()),
        ] {
            if value.is_empty() {
                bail!("`comments.{field}` cannot be empty");
            }
        }

        Ok(self)
    }
}
