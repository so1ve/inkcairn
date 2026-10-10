use std::fmt::{self, Write as _};

use comrak::html::escape;
use syntaxmate::{FontModifiers, RgbColor, Style, Theme, TokenizedDocument, TokenizedLine};

pub struct Highlighter {
    engine: syntaxmate::Highlighter,
    light_theme: Theme,
    dark_theme: Theme,
}

impl Default for Highlighter {
    fn default() -> Self {
        Self {
            engine: syntaxmate::Highlighter::bundled().unwrap(),
            light_theme: Theme::bundled("github-light").unwrap(),
            dark_theme: Theme::bundled("github-dark").unwrap(),
        }
    }
}

impl Highlighter {
    pub fn highlight(&self, language: Option<&str>, code: &str) -> Option<TokenizedDocument> {
        let language = language?.to_ascii_lowercase();
        self.engine.tokenize(&language, code).ok()
    }

    pub fn write_line(
        &self,
        output: &mut String,
        source: &str,
        highlighted: &TokenizedLine,
    ) -> fmt::Result {
        let mut cursor = 0;
        for token in highlighted.tokens() {
            let range = token.range();
            let start = range.start.min(source.len());
            let end = range.end.min(source.len());
            if start < cursor || end < start {
                continue;
            }

            escape(output, &source[cursor..start])?;
            let style = InlineStyle {
                light: self.light_theme.resolve(token.scopes()),
                dark: self.dark_theme.resolve(token.scopes()),
            };
            write!(output, "<span style=\"{style}\">")?;
            escape(output, &source[start..end])?;
            output.push_str("</span>");
            cursor = end;
        }
        escape(output, &source[cursor..])
    }
}

struct InlineStyle {
    light: Style,
    dark: Style,
}

impl fmt::Display for InlineStyle {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        match (self.light.foreground, self.dark.foreground) {
            (Some(light), Some(dark)) => {
                write!(
                    output,
                    "color:light-dark({},{});",
                    CssColor(light),
                    CssColor(dark)
                )?;
            }
            (Some(color), None) | (None, Some(color)) => {
                write!(output, "color:{};", CssColor(color))?;
            }
            (None, None) => {}
        }
        if self.light.modifiers == self.dark.modifiers {
            write_modifiers(output, self.light.modifiers)?;
        }
        Ok(())
    }
}

struct CssColor(RgbColor);

impl fmt::Display for CssColor {
    fn fmt(&self, output: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            output,
            "#{:02x}{:02x}{:02x}",
            self.0.red, self.0.green, self.0.blue
        )
    }
}

fn write_modifiers(output: &mut fmt::Formatter<'_>, modifiers: FontModifiers) -> fmt::Result {
    if modifiers.contains(FontModifiers::BOLD) {
        output.write_str("font-weight:bold;")?;
    }
    if modifiers.contains(FontModifiers::ITALIC) {
        output.write_str("font-style:italic;")?;
    }
    match (
        modifiers.contains(FontModifiers::UNDERLINED),
        modifiers.contains(FontModifiers::CROSSED_OUT),
    ) {
        (true, true) => output.write_str("text-decoration:underline line-through;"),
        (true, false) => output.write_str("text-decoration:underline;"),
        (false, true) => output.write_str("text-decoration:line-through;"),
        (false, false) => Ok(()),
    }
}
