use askama::Template;

use super::{Navigation, PageContext, Site};

#[derive(Template)]
#[template(path = "not-found.html")]
struct NotFoundTemplate<'a, 'site> {
    page: PageContext<'a, 'site>,
    home_href: String,
}

impl<'site> Site<'site> {
    pub fn page<'a>(&'a self, path: &str, title: &str) -> PageContext<'a, 'site> {
        let title = if path.is_empty() {
            self.metadata.title.clone()
        } else {
            format!("{title} — {}", self.metadata.title)
        };
        let canonical_url = self
            .metadata
            .url
            .as_deref()
            .map(|url| format!("{url}/{path}"));

        let mut navigation = Vec::with_capacity(self.pages.len() + 3);

        navigation.push(Navigation {
            href: self.href(""),
            label: "Home",
            current: path.is_empty(),
        });
        navigation.push(Navigation {
            href: self.href("posts.html"),
            label: "Posts",
            current: path == "posts.html" || path.starts_with("posts/"),
        });
        navigation.push(Navigation {
            href: self.href("archive.html"),
            label: "Archive",
            current: path == "archive.html",
        });
        navigation.extend(self.pages.iter().map(|page| Navigation {
            href: self.href(&page.path.url),
            label: &page.article.title.html,
            current: path == page.path.url,
        }));

        PageContext {
            site: self,
            title,
            canonical_url,
            noindex: false,
            navigation,
        }
    }

    pub fn not_found(&self) -> String {
        let mut page = self.page("404.html", "Page not found");
        page.canonical_url = None;

        NotFoundTemplate {
            page,
            home_href: self.href(""),
        }
        .render()
        .unwrap()
    }
}
