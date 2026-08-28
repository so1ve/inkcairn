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

        output.write_str("<span class=\"code-line")?;
        output.write_str(class)?;
        output.write_str("\">")?;
        let mut html = String::new();
        match highlighted
            .as_ref()
            .and_then(|document| document.lines().get(index))
        {
            Some(highlighted) => highlighter.write_line(&mut html, line, highlighted)?,
            None => escape(&mut html, line)?,
        }
        output.write_str(&html)?;
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
        " diff diff-add"
    } else if line.starts_with('-') {
        " diff diff-del"
    } else if line.starts_with("@@") {
        " diff diff-header"
    } else {
        " diff"
    }
}
