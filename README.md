# Inkcairn

Simple blog generator using your Git index.

Inkcairn turns a Git repository of Markdown files into a complete static blog.
It provides a built-in layout, pages, categories, syntax highlighting, search,
feeds, and live preview, then writes plain static files that can be hosted
anywhere. Publication and update dates come from file history by default, so
authors normally write no date metadata.

## Quick start

Create and preview a site:

```sh
inkcairn init my-blog
inkcairn dev my-blog
```

For a new site, `init` also creates the Git repository and its first commit.
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
# Hello

A short introduction to the post.

## First section

Write the rest of the post here.
```

The first `#` heading is the title. The introductory content is shown in post
lists.

### Dates

Do not normally add a date to a post. For a committed file, Inkcairn uses its
first Git commit as the publication date and its latest commit as the update
date. If the file is dirty, its filesystem modification date becomes the update
date. Untracked and non-Git files use filesystem creation and modification
dates during preview or an `--allow-dirty` build.

Use `published` only when importing an older post whose original publication
date is not represented by its Git history:

```markdown
---
published: 2020-04-12
---

# An older post
```

The value must use `YYYY-MM-DD`. Setting it overrides only the publication
date; the update date still follows the latest Git or filesystem change.

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

Render a responsive list of friend links with a `friends` block:

````markdown
```friends
- name: John Doe
  url: https://john.example.com
  description: A short description
  avatar: /assets/john.png

- name: Jane Smith
  url: https://jane.example.com
```
````

`name` and `url` are required. `description` and `avatar` are optional. Without
an avatar, the card displays the first character of its name. Entries retain
their order in the Markdown file.

## Add assets and snippets

Place images and other static files in `assets/`. Reference them from Markdown
with paths such as `/assets/photo.jpg`.

Optional snippets can add shared content:

```text
snippets/head.html         content added to every page's <head>
snippets/home.md           content shown above the home-page post list
snippets/after-content.md  content shown after every post and page
```

Each snippet may use either `.md` or `.html`, but not both.

## Preview and build

Preview on the default port:

```sh
inkcairn dev
```

Choose another port:

```sh
inkcairn dev --port 8080
```

Create a production build in `dist/`:

```sh
inkcairn build
```

A normal build requires a clean Git worktree. To build uncommitted, untracked,
or non-Git content:

```sh
inkcairn build --allow-dirty
```

To include files ending in `.draft.md`:

```sh
inkcairn build --include-drafts
```

## Credits

Theme is based on [Cactus](https://github.com/probberechts/hexo-theme-cactus)

## LICENSE

MIT. Made with ❤️ by [Ray](https://github.com/so1ve)
