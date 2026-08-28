use lightningcss::printer::PrinterOptions;
use lightningcss::stylesheet::{MinifyOptions, ParserOptions, StyleSheet};

pub fn render() -> String {
    minify(include_str!("../../theme/style.css"), "style.css")
}

pub fn search() -> String {
    minify(include_str!("../../theme/search.css"), "search.css")
}

pub fn comments() -> String {
    minify(include_str!("../../theme/comments.css"), "comments.css")
}

fn minify(source: &str, filename: &str) -> String {
    let mut stylesheet = StyleSheet::parse(
        source,
        ParserOptions {
            filename: filename.to_owned(),
            ..ParserOptions::default()
        },
    )
    .unwrap();
    stylesheet.minify(MinifyOptions::default()).unwrap();
    let css = stylesheet
        .to_css(PrinterOptions {
            minify: true,
            ..PrinterOptions::default()
        })
        .unwrap();

    css.code
}
