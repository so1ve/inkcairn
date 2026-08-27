use percent_encoding::{AsciiSet, NON_ALPHANUMERIC, utf8_percent_encode};

const PATH_SEGMENT: &AsciiSet = &NON_ALPHANUMERIC
    .remove(b'-')
    .remove(b'.')
    .remove(b'_')
    .remove(b'~');

pub fn encode(path: &str) -> String {
    path.split('/')
        .map(|segment| utf8_percent_encode(segment, PATH_SEGMENT).to_string())
        .collect::<Vec<_>>()
        .join("/")
}

pub fn base(url: Option<&str>) -> String {
    let Some(url) = url else {
        return String::new();
    };
    let (_, rest) = url.split_once("://").unwrap();

    match rest.find('/') {
        Some(index) => rest[index..].trim_end_matches('/').to_owned(),
        None => String::new(),
    }
}
