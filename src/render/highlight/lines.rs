use std::fmt;

use comrak::html::escape;
use syntect::html::{ClassStyle, line_tokens_to_classed_spans};
use syntect::parsing::{ParseState, ScopeStack, SyntaxReference, SyntaxSet};
use syntect::util::LinesWithEndings;

pub fn write(
    output: &mut dyn fmt::Write,
    syntaxes: &SyntaxSet,
    syntax: &SyntaxReference,
    code: &str,
    diff: bool,
) -> fmt::Result {
    let mut highlighter = LineHighlighter::new(syntaxes, syntax);

    for (index, line) in LinesWithEndings::from(code).enumerate() {
        let (class, line) = if diff {
            if let Some(content) = line.strip_prefix('+') {
                (" diff-add", content)
            } else if let Some(content) = line.strip_prefix('-') {
                (" diff-del", content)
            } else if line.starts_with("@@") {
                (" diff-header", line)
            } else {
                ("", line)
            }
        } else {
            ("", line)
        };
        let indent = line
            .bytes()
            .take_while(|byte| matches!(byte, b' ' | b'\t'))
            .fold(0, |column, byte| {
                if byte == b'\t' {
                    column + 2 - column % 2
                } else {
                    column + 1
                }
            });

        write!(
            output,
            "<span class=\"code-line-number{class}\" aria-hidden=\"true\">{}</span><span class=\"code-text{class}\" style=\"--indent:{indent}ch\">",
            index + 1
        )?;
        match highlighter.highlight(line) {
            Ok(html) => output.write_str(&html)?,
            Err(_) => escape(output, line.trim_end_matches(['\r', '\n']))?,
        }
        output.write_str("</span>")?;
    }

    Ok(())
}

struct LineHighlighter<'a> {
    syntaxes: &'a SyntaxSet,
    parser: ParseState,
    scopes: ScopeStack,
}

impl<'a> LineHighlighter<'a> {
    fn new(syntaxes: &'a SyntaxSet, syntax: &SyntaxReference) -> Self {
        Self {
            syntaxes,
            parser: ParseState::new(syntax),
            scopes: ScopeStack::new(),
        }
    }

    fn highlight(&mut self, line: &str) -> Result<String, syntect::Error> {
        let mut html = String::new();
        for scope in &self.scopes.scopes {
            html.push_str("<span class=\"syn-");
            html.push_str(&scope.build_string().replace('.', " syn-"));
            html.push_str("\">");
        }

        let operations = self.parser.parse_line(line, self.syntaxes)?;
        let (highlighted, _) = line_tokens_to_classed_spans(
            line,
            &operations,
            ClassStyle::SpacedPrefixed { prefix: "syn-" },
            &mut self.scopes,
        )?;
        html.push_str(&highlighted);

        if line.ends_with('\n') {
            let newline = html.rfind('\n').unwrap();
            html.remove(newline);
            if newline > 0 && html.as_bytes()[newline - 1] == b'\r' {
                html.remove(newline - 1);
            }
        }

        for _ in &self.scopes.scopes {
            html.push_str("</span>");
        }

        Ok(add_break_opportunities(&html))
    }
}

fn add_break_opportunities(html: &str) -> String {
    let mut output = String::with_capacity(html.len());
    let mut characters = html.chars().peekable();
    let mut previous = None;
    let mut in_tag = false;

    while let Some(character) = characters.next() {
        if in_tag {
            output.push(character);
            if character == '>' {
                in_tag = false;
            }
            continue;
        }

        if character == '<' {
            in_tag = true;
            output.push(character);
            continue;
        }

        if character == ':' && characters.peek() == Some(&':') {
            output.push_str("::<wbr>");
            characters.next();
            previous = Some(':');
            continue;
        }

        if character == '.'
            && previous != Some('.')
            && !(previous.is_some_and(|value: char| value.is_ascii_digit())
                && characters.peek().is_some_and(char::is_ascii_digit))
        {
            output.push_str("<wbr>");
        }

        output.push(character);

        if character == ',' {
            output.push_str("<wbr>");
        }
        previous = Some(character);
    }

    output
}
