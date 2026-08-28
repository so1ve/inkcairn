use std::borrow::Cow;
use std::fmt;

use comrak::html::escape;

use super::syntax::Highlighter;

pub fn write(
    output: &mut dyn fmt::Write,
    highlighter: &Highlighter,
    language: Option<&str>,
    source: &str,
    diff: bool,
) -> fmt::Result {
    let code = code_without_diff_markers(source, diff);
    let highlighted = highlighter.highlight(language, &code);
    let mut source_lines = source.split_inclusive('\n');

    for (index, line) in code.split_inclusive('\n').enumerate() {
        let source_line = source_lines.next().unwrap();
        let class = diff_class(source_line, diff);
        let line = line.trim_end_matches(['\r', '\n']);
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
        let mut html = String::new();
        match highlighted
            .as_ref()
            .and_then(|document| document.lines().get(index))
        {
            Some(highlighted) => highlighter.write_line(&mut html, line, highlighted)?,
            None => escape(&mut html, line)?,
        }
        output.write_str(&add_break_opportunities(&html))?;
        output.write_str("</span>")?;
    }

    Ok(())
}

fn code_without_diff_markers(code: &str, diff: bool) -> Cow<'_, str> {
    if !diff {
        return Cow::Borrowed(code);
    }

    Cow::Owned(
        code.split_inclusive('\n')
            .map(|line| line.strip_prefix(['+', '-']).unwrap_or(line))
            .collect(),
    )
}

fn diff_class(line: &str, diff: bool) -> &'static str {
    if !diff {
        ""
    } else if line.starts_with('+') {
        " diff-add"
    } else if line.starts_with('-') {
        " diff-del"
    } else if line.starts_with("@@") {
        " diff-header"
    } else {
        ""
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
