use askama::Template;

use super::{PageContext, Site};

#[derive(Template)]
#[template(path = "search.html")]
struct SearchTemplate<'a, 'site> {
    page: PageContext<'a, 'site>,
}

impl Site<'_> {
    pub fn search_page(&self) -> String {
        SearchTemplate {
            page: self.page("search.html", "Search"),
        }
        .render()
        .unwrap()
    }
}
