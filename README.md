# Inkcairn

A simple static blog generator for Markdown.

## Quick start

Create and preview a site:

```sh
inkcairn init my-blog
inkcairn dev my-blog
```

Open <http://127.0.0.1:3201/> and start editing files in `my-blog/`. The preview
updates when the site changes.

When the site is ready to publish:

```sh
inkcairn build my-blog
```

Upload the contents of `my-blog/dist/` to any static hosting service.

## Configure the site

Edit `inkcairn.md`:

```markdown
---
url: https://example.com
language: en
---

# My Blog

Notes on software and writing.
```

The first `#` heading is the site title, and the introductory text is its
description. Only the title is required. `language` defaults to `en`. Set `url`
when publishing the site so links, the RSS feed, and the sitemap use the public
address.

## Write posts

Add Markdown files to `posts/`:

```markdown
---
published: 2026-08-27
---

# Hello

A short introduction to the post.

## First section

Write the rest of the post here.
```

The first `#` heading is the title. The introductory content is shown in post
lists. The `published` field is optional and must use `YYYY-MM-DD`.

The filename becomes the URL:

```text
posts/hello.md              -> /posts/hello.html
posts/01-hello.md           -> /posts/hello.html
posts/notes/hello.md        -> /posts/notes/hello.html
```

A leading two-digit prefix such as `01-` is omitted from the URL. Nested
directories create post categories. Use `__` in a category directory name where
its displayed name should contain a space.

Append `.draft` to keep a post out of normal builds:

```text
posts/next-post.draft.md
```

## Add pages

Add Markdown files to `pages/`. Pages use the same format as posts and are
generated at the site root:

```text
pages/01-about.md     -> /about.html
pages/02-projects.md  -> /projects.html
```

The numeric prefixes also set the navigation order.

## Use Markdown features

Fenced code blocks support syntax highlighting:

````markdown
```rust
fn main() {
    println!("Hello");
}
```
````

Add `diff` after the language to highlight changed lines:

````markdown
```rust,diff
-let published = false;
+let published = true;
```
````

Footnotes and callouts are also available:

```markdown
This sentence has a note.[^note]

[^note]: Footnote text.

> [!TIP]
> Callouts can be Note, Tip, Important, Warning, or Caution.
```

## Add assets and snippets

Place images and other static files in `assets/`. Reference them from Markdown
with paths such as `/assets/photo.jpg`.

Optional snippets can add shared content:

```text
snippets/head.html       content added to every page's <head>
snippets/home.md         content shown above the home-page post list
snippets/after-content.md content shown after every post and page
```

Each snippet may use either `.md` or `.html`, but not both.

## Preview and build

Preview on the default port:

```sh
inkcairn dev my-blog
```

Choose another port:

```sh
inkcairn dev --port 8080 my-blog
```

Create a production build in `dist/`:

```sh
inkcairn build my-blog
```

A normal build requires a clean Git worktree. To build uncommitted, untracked,
or non-Git content:

```sh
inkcairn build --allow-dirty my-blog
```

To include files ending in `.draft.md`:

```sh
inkcairn build --include-drafts my-blog
```
