use lightningcss::printer::PrinterOptions;
use lightningcss::stylesheet::{MinifyOptions, ParserOptions, StyleSheet};
use syntect::highlighting::ThemeSet;
use syntect::html::{ClassStyle, css_for_theme_with_class_style};

pub fn render() -> String {
    let themes = ThemeSet::load_defaults();
    let class_style = ClassStyle::SpacedPrefixed { prefix: "syn-" };
    let mut light = themes.themes.get("InspiredGitHub").unwrap().clone();
    light.settings.background = None;
    let light = css_for_theme_with_class_style(&light, class_style).unwrap();
    let mut dark = themes.themes.get("base16-ocean.dark").unwrap().clone();
    dark.settings.background = None;
    let dark = css_for_theme_with_class_style(&dark, class_style).unwrap();
    let syntax_css = format!(
        "{}\n{}",
        prefix_syntax_css(&light, ".markdown-body "),
        prefix_syntax_css(&dark, ":root[data-theme=\"dark\"] .markdown-body ")
    );
    let source = format!(
        "{}\n{}\n",
        include_str!("../../theme/style.css"),
        syntax_css
    );

    minify(&source, "style.css")
}

pub fn search() -> String {
    minify(include_str!("../../theme/search.css"), "search.css")
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

fn prefix_syntax_css(css: &str, prefix: &str) -> String {
    css.lines()
        .map(|line| {
            if line.starts_with('.') {
                format!("{prefix}{line}")
            } else {
                line.to_owned()
            }
        })
        .collect::<Vec<_>>()
        .join("\n")
}
