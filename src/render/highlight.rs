mod lines;

use std::borrow::Cow;
use std::collections::HashMap;
use std::fmt;

use comrak::adapters::SyntaxHighlighterAdapter;
use comrak::html::write_opening_tag;
use syntect::parsing::SyntaxSet;

const COPY_BUTTON: &str = r#"<button class="copy-code" type="button" aria-label="Copy code" title="Copy code"><svg class="copy-icon" viewBox="0 0 16 16" aria-hidden="true"><g transform="translate(1 1)"><rect x="5" y="5" width="8" height="8" rx="1"></rect><path d="M3 11H2a1 1 0 0 1-1-1V2a1 1 0 0 1 1-1h8a1 1 0 0 1 1 1v1"></path></g></svg><svg class="copy-success" viewBox="0 0 16 16" aria-hidden="true"><path d="m2.5 8 3 3 8-8"></path></svg></button>"#;

pub struct Highlighter {
    syntaxes: SyntaxSet,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self {
            syntaxes: two_face::syntax::extra_newlines(),
        }
    }
}

impl SyntaxHighlighterAdapter for Highlighter {
    fn write_highlighted(
        &self,
        output: &mut dyn fmt::Write,
        info: Option<&str>,
        code: &str,
    ) -> fmt::Result {
        let fence = Fence::parse(info);
        let syntax = fence
            .language
            .and_then(|language| {
                let language = match language {
                    "csharp" => "cs",
                    "docker" => "dockerfile",
                    "jsx" => "tsx",
                    "jsonc" => "json",
                    "shell" => "bash",
                    _ => language,
                };
                self.syntaxes.find_syntax_by_token(language)
            })
            .unwrap_or_else(|| self.syntaxes.find_syntax_plain_text());

        lines::write(output, &self.syntaxes, syntax, code, fence.diff)
    }

    fn write_pre_tag(
        &self,
        output: &mut dyn fmt::Write,
        mut attributes: HashMap<&'static str, Cow<'_, str>>,
    ) -> fmt::Result {
        if let Some(info) = attributes.remove("lang") {
            let label = Fence::parse(Some(info.as_ref())).label();
            attributes.insert("data-meta", Cow::Owned(label));
        }

        write_opening_tag(output, "pre", attributes)?;
        output.write_str(COPY_BUTTON)
    }

    fn write_code_tag(
        &self,
        output: &mut dyn fmt::Write,
        _attributes: HashMap<&'static str, Cow<'_, str>>,
    ) -> fmt::Result {
        output.write_str("<code>")
    }
}

struct Fence<'a> {
    language: Option<&'a str>,
    metadata: Vec<&'a str>,
    diff: bool,
}

impl<'a> Fence<'a> {
    fn parse(info: Option<&'a str>) -> Self {
        let mut tokens = info
            .into_iter()
            .flat_map(|info| info.split(','))
            .map(str::trim)
            .filter(|token| !token.is_empty());
        let language = tokens.next();
        let metadata: Vec<_> = tokens.collect();
        let diff = metadata.contains(&"diff");

        Self {
            language,
            metadata,
            diff,
        }
    }

    fn label(&self) -> String {
        self.language
            .into_iter()
            .chain(self.metadata.iter().copied())
            .collect::<Vec<_>>()
            .join(" · ")
    }
}
