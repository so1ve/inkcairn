# Vellum Product Architecture Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Build Vellum, a small, opinionated, Markdown-first static blog generator with excellent defaults, Git-aware metadata, syntax-highlighted code blocks, deterministic output, and minimal configuration.

**Architecture:** Vellum has one rendering path and one Markdown engine. It uses Comrak for Markdown parsing, Vellum-owned renderers for enhanced blocks and code fences, Askama for the fixed built-in page shell, Rayon for parallel generation, and Git CLI data for dates/source provenance. It does not expose themes, templates, plugins, categories, or multiple renderers.

**Tech Stack:** Rust, Comrak, Syntect, Askama, Rayon, Git CLI, optional future `ureq` for explicit external fetches, optional hash/cache layer for manifests and external data.

---

## Product Positioning

**Name:** Vellum

**Tagline:** Write first. Vellum handles the rest.

**Short description:** Vellum is a Markdown-first blog generator for durable writing. Authors write plain Markdown; Vellum handles clean pages, tags, feeds, archives, Git-derived dates, source links, syntax highlighting, and reproducible build metadata with minimal configuration.

Vellum is not a static-site framework. It is an opinionated writing tool.

### Product Principles

1. **One house style.** No theme system in v1.
2. **One Markdown engine.** No dual renderer, no fallback parser, no parser abstraction for hypothetical swaps.
3. **Markdown-first.** Markdown is the native content language.
4. **Git-aware, not Git-centered.** Git automatically fills dates, source links, commit footers, and build provenance, but the product remains content-first.
5. **Small by product surface.** Avoid themes, plugins, category systems, frontend build chains, JS frameworks, search WASM, i18n frameworks, and user template APIs.
6. **Syntax-aware from day one.** Code block highlighting and diff rendering are part of the MVP architecture, not a later bolt-on.
7. **Parallel by design.** Rendering should be page-parallel from the first implementation.
8. **Network is explicit.** Normal builds must not make HTTP requests. External data is fetched only through explicit commands or flags and recorded as input.

---

## MVP Scope

### In Scope

- Markdown posts and pages.
- Minimal site configuration.
- Tags only; no categories.
- One built-in house style.
- Raw HTML allowed in Markdown.
- Fixed snippets:
  - `snippets/head.html`
  - `snippets/after-post.html`
- Comrak block directives for built-in enhanced blocks.
- Syntax-highlighted fenced code blocks.
- Diff-style code blocks:
  - ````markdown
    ```diff
    - old
    + new
    ```
    ````
  - ````markdown
    ```rust,diff
    - let x = 1;
    + let x = 2;
    ```
    ````
- Git-derived published/updated dates.
- Build commit footer with origin commit link when origin is recognized.
- Dirty workspace policy.
- RSS feed.
- Sitemap.
- Build manifest.
- Parallel page rendering.

### Out of Scope for MVP

- Theme system.
- User templates.
- Plugin system.
- Categories.
- Search index.
- WASM search.
- Typst support.
- Markdown renderer swaps.
- Inline code syntax highlighting.
- Code line highlighting.
- Code line numbers.
- Tailwind, Sass, PostCSS, Vite, React, Leptos, or other frontend build chains.
- Git library bindings such as `git2`.
- Async runtime such as Tokio.
- Default HTTP fetching during build.
- Built-in comments provider configuration.

---

## Default Project Layout

```text
my-blog/
  vellum.toml
  posts/
    2026-07-01-hello.md
  pages/
    about.md
  assets/
    favicon.svg
  snippets/
    head.html
    after-post.html
```

Only `posts/` is strictly required for a useful local build. Public-site builds should provide `title` and `url`.

---

## Configuration Policy

Default `vellum.toml` should be tiny:

```toml
title = "My Blog"
url = "https://example.com"
```

Optional fields:

```toml
description = "Notes on software and writing."
language = "zh-CN"
```

Do not add config for features that can be derived or convention-based.

### Do Not Add

```toml
theme = "..."
category = "..."
categories = []
taxonomies = []
menu = []
comments = {}
params = {}
date_format = "..."
paginate = 10
```

### Rules

- RSS and sitemap are generated automatically when `url` is present.
- Tags are generated only if posts have tags.
- Git origin is auto-detected.
- Build commit footer is enabled by default when Git metadata is available.
- Source link can have a future escape hatch, but should not appear in the default config.
- Unknown config keys should be hard errors with suggestions.

---

## Markdown Syntax Policy

Vellum uses **Comrak only**.

```toml
comrak = { version = "0.52", default-features = false }
```

Rationale:

- Comrak gives Vellum a real AST.
- Comrak supports block directive style syntax.
- Comrak code fence info strings are available for Vellum-owned code rendering.
- Comrak makes link rewriting, heading IDs, raw HTML, and AST-level analysis more natural.
- Vellum avoids Comrak default features to avoid pulling CLI/syntect/extra functionality.

### Raw HTML

Raw HTML in Markdown is allowed. This is a personal publishing tool; authors control their own content.

No sanitizer in MVP.

### Block Directives

Use Comrak-style block directives for built-in enhanced blocks:

```markdown
:::note
This is a note.
:::
```

```markdown
:::warning
Be careful.
:::
```

```markdown
:::github rust-lang/rust
The Rust programming language.
:::
```

MVP can ship with only `github`, `note`, and `warning` if implementation time is constrained.

No user-defined directives in v1.

---

## Code Block Rendering Policy

Vellum owns code block rendering.

Use Syntect for token coloring:

```toml
syntect = { version = "5", default-features = false, features = ["default-fancy"] }
```

Rationale:

- Pure Rust path; no Oniguruma C dependency.
- Mature and battle-tested.
- Good enough language coverage for blogs.
- Deterministic output.
- Integrates cleanly with a custom code block renderer.

### Supported MVP Fence Metadata

```markdown
```rust
fn main() {}
```
```

```markdown
```diff
- old
+ new
```
```

```markdown
```rust,diff
- let x = 1;
+ let x = 2;
```
```

### Not Supported in MVP

```markdown
```rust,hl_lines=1 3-5
```

```markdown
```rust,linenos
```

```markdown
`Vec<T>`{rust}
```

No inline syntax highlighting in MVP. Inline code receives house-style visual styling only.

### Fence Parser

```rust
struct CodeFence {
    language: Option<String>,
    diff: bool,
}
```

Parsing rules:

- Split info string by commas.
- First non-flag token is language.
- `diff` is a flag.
- `diff` alone means diff mode with no inner language.
- `rust,diff` means diff line semantics with Rust token highlighting.
- Unknown flags produce warnings, not hard errors.

### Diff Rendering

For `rust,diff`, Vellum should:

1. Detect line prefix:
   - `+` -> added
   - `-` -> removed
   - space -> context
   - `@@` -> hunk header
2. Preserve the marker in output.
3. Highlight the code content after the marker using the declared language.
4. Add line-level classes:
   - `diff-add`
   - `diff-del`
   - `diff-context`
   - `diff-header`

For plain `diff`, Vellum can initially apply line-level classes without inner language token highlighting.

---

## Rendering Architecture

One path:

```text
Markdown source
  -> frontmatter parse
  -> Comrak parse
  -> Vellum directive/code render hooks
  -> article HTML
  -> Askama page shell
  -> output file
```

Askama is internal only:

```toml
askama = "0.16"
```

No user-facing template files. Templates are implementation details compiled into Vellum.

Suggested internal templates:

```text
templates/
  base.html
  index.html
  post.html
  page.html
  tag.html
  archive.html
```

---

## Parallel Build Architecture

Use Rayon:

```toml
rayon = "1"
```

No Tokio. No async runtime.

### Build Phases

```text
Discover
  -> Parse/analyze all content
  -> Build GitIndex
  -> Collect external needs
  -> Resolve external data from cache or explicit fetch
  -> Render posts/pages in parallel
  -> Aggregate index/tag/archive/feed/sitemap
  -> Write files
  -> Write manifest
```

### Core Context

```rust
struct BuildContext {
    site: SiteConfig,
    git: GitIndex,
    snippets: Snippets,
    external: ExternalStore,
    highlighter: Highlighter,
}
```

Page rendering should be embarrassingly parallel:

```rust
posts.par_iter()
    .map(|post| render_post(post, &context))
    .collect::<Result<Vec<_>>>()
```

Do not introduce a DAG library in MVP unless concrete dependencies require it. The pipeline phases are enough.

---

## Git Architecture

Use Git CLI, not `git2`.

Build a `GitIndex` once per build:

```rust
struct GitIndex {
    head: String,
    short_head: String,
    origin: Option<RemoteOrigin>,
    dirty: bool,
    files: HashMap<PathBuf, FileGitInfo>,
}

struct FileGitInfo {
    created_at: Option<String>,
    updated_at: Option<String>,
    created_commit: Option<String>,
    updated_commit: Option<String>,
}
```

Required Git data:

- `git rev-parse HEAD`
- `git remote get-url origin`
- `git status --porcelain`
- per-file first commit date
- per-file last commit date

Remote origin detection should support at least:

- GitHub
- GitLab
- Codeberg

Footer text should be content-first and unobtrusive, for example:

```text
Generated from commit abc1234
```

with the hash linking to the remote commit URL when available.

---

## External Data Architecture

External HTTP should be architecture-ready, but not required in MVP.

Normal builds must not make network requests.

Future commands:

```bash
vellum fetch
vellum build --fetch
vellum build --offline
```

Future optional dependencies:

```toml
ureq = { version = "3", optional = true }
serde_json = { version = "1", optional = true }
```

No `reqwest`, no Tokio.

### GitHub Card Behavior

MVP can render static cards without API calls:

```markdown
:::github rust-lang/rust
The Rust programming language.
:::
```

Future fetch mode can enrich with:

- description
- stars
- language
- updated time

External data must be treated as input:

- cache it
- hash it
- record it in the manifest

---

## Manifest

Manifest is a first-class build artifact.

```text
public/manifest.json
```

It should record:

- Vellum version
- source commit
- source remote
- dirty status
- generated page hashes
- source file mapping
- external data hashes when external data exists

Example shape:

```json
{
  "generator": "vellum 0.1.0",
  "source": {
    "commit": "abc123...",
    "remote": "https://github.com/user/blog",
    "dirty": false
  },
  "pages": {
    "/posts/hello/": {
      "source": "posts/2026-07-01-hello.md",
      "hash": "blake3:..."
    }
  }
}
```

Hash crate candidate:

```toml
blake3 = "1"
```

---

## Implementation Tasks

### Task 1: Create Rust Project Skeleton

**Files:**
- Create: `Cargo.toml`
- Create: `src/main.rs`
- Create: `src/lib.rs`

**Step 1: Initialize crate structure**

Create a binary crate named `vellum`.

**Step 2: Add minimal dependencies**

Start with:

```toml
[dependencies]
comrak = { version = "0.52", default-features = false }
askama = "0.16"
rayon = "1"
```

Add `syntect` only when implementing Task 6.

**Step 3: Add smoke test command**

Run:

```bash
cargo check
```

Expected: crate compiles.

---

### Task 2: Implement Minimal Config Loading

**Files:**
- Create: `src/config.rs`
- Modify: `src/lib.rs`

**Step 1: Define config**

```rust
pub struct SiteConfig {
    pub title: String,
    pub url: Option<String>,
    pub description: Option<String>,
    pub language: Option<String>,
}
```

**Step 2: Parse `vellum.toml`**

Use `toml` and `serde` only if hand parsing becomes annoying. If added:

```toml
serde = { version = "1", features = ["derive"] }
toml = "0.8"
```

**Step 3: Reject unknown fields**

Config must be strict.

---

### Task 3: Discover Content Files

**Files:**
- Create: `src/content/discover.rs`
- Create: `src/content/mod.rs`

**Step 1: Find posts and pages**

Rules:

- `posts/**/*.md` -> posts
- `pages/**/*.md` -> pages

**Step 2: Derive slug**

Strip leading date prefix from filenames:

```text
2026-07-01-hello.md -> hello
```

**Step 3: Preserve source path**

Every discovered item must keep its source path for Git metadata and diagnostics.

---

### Task 4: Parse Frontmatter and Metadata

**Files:**
- Create: `src/content/frontmatter.rs`
- Modify: `src/content/mod.rs`

**Step 1: Implement tiny frontmatter parser**

Support only:

```yaml
title: My Post
description: Optional description
tags: rust, writing
draft: true
slug: custom-slug
date: 2026-07-01
```

**Step 2: Do not pull in YAML parser**

Vellum frontmatter is a small Vellum subset, not YAML.

**Step 3: Fallback rules**

- title: frontmatter title -> first H1 -> filename
- slug: frontmatter slug -> filename slug
- date: frontmatter date -> Git created date -> none
- tags: frontmatter tags -> empty

---

### Task 5: Parse Markdown with Comrak

**Files:**
- Create: `src/content/markdown.rs`

**Step 1: Configure Comrak**

Enable required extensions only:

- table
- strikethrough
- tasklist
- autolink
- maybe alerts
- block directives

Do not enable everything blindly.

**Step 2: Extract title and links from AST**

Walk AST for:

- first heading level 1
- links
- code blocks
- block directives

**Step 3: Allow raw HTML**

Render options must allow raw HTML pass-through.

---

### Task 6: Implement Code Block Renderer

**Files:**
- Create: `src/render/highlight.rs`
- Modify: `src/content/markdown.rs`

**Step 1: Add Syntect dependency**

```toml
syntect = { version = "5", default-features = false, features = ["default-fancy"] }
```

**Step 2: Parse fence info**

Implement:

```rust
struct CodeFence {
    language: Option<String>,
    diff: bool,
}
```

**Step 3: Render normal code**

Use Syntect to produce highlighted HTML.

**Step 4: Render diff blocks**

Support:

```markdown
```diff
- old
+ new
```
```

and:

```markdown
```rust,diff
- let x = 1;
+ let x = 2;
```
```

**Step 5: Do not implement line highlight or line numbers**

Keep v1 focused.

---

### Task 7: Implement Built-In Directives

**Files:**
- Create: `src/content/directives.rs`
- Modify: `src/content/markdown.rs`
- Modify: `src/theme/style.css`

**Step 1: Support note/warning**

```markdown
:::note
Hello
:::
```

```markdown
:::warning
Careful
:::
```

**Step 2: Support GitHub card**

```markdown
:::github rust-lang/rust
The Rust programming language.
:::
```

**Step 3: No network calls**

MVP GitHub card is static only.

---

### Task 8: Build Askama Page Shell

**Files:**
- Create: `templates/base.html`
- Create: `templates/post.html`
- Create: `templates/page.html`
- Create: `templates/index.html`
- Create: `templates/tag.html`
- Create: `templates/archive.html`
- Create: `src/render/templates.rs`

**Step 1: Create one house style**

No theme parameter. No external templates.

**Step 2: Inject snippets**

- `snippets/head.html` into head
- `snippets/after-post.html` after article body

**Step 3: Render Git footer**

Show build commit when available.

---

### Task 9: Implement GitIndex

**Files:**
- Create: `src/git/index.rs`
- Create: `src/git/origin.rs`
- Create: `src/git/mod.rs`

**Step 1: Collect repository state**

Use Git CLI:

```bash
git rev-parse HEAD
git remote get-url origin
git status --porcelain
```

**Step 2: Resolve per-file dates**

Use Git log commands and cache results in memory.

**Step 3: Parse origin URL**

Support GitHub/GitLab/Codeberg commit URL generation.

---

### Task 10: Implement Parallel Build

**Files:**
- Create: `src/build/context.rs`
- Create: `src/build/mod.rs`
- Create: `src/build/writer.rs`

**Step 1: Build context once**

Context includes config, snippets, GitIndex, and highlighter.

**Step 2: Render posts/pages with Rayon**

Use `par_iter()`.

**Step 3: Aggregate global pages after render**

Generate:

- index
- tags
- archive
- RSS
- sitemap

---

### Task 11: Implement Manifest

**Files:**
- Create: `src/build/manifest.rs`

**Step 1: Hash generated pages**

Use `blake3` if accepted:

```toml
blake3 = "1"
```

**Step 2: Record source commit**

Include Git HEAD and dirty status.

**Step 3: Write `public/manifest.json`**

Manifest should be stable and deterministic.

---

## Open Decisions

1. Whether `serde` + `toml` is acceptable for site config or if config should be hand-parsed.
2. Whether RSS XML should be hand-written or use a small XML crate.
3. Whether `syntect` is default-on in official binary or cargo-feature gated with default enabled.
4. Whether Comrak alerts extension should be enabled, or whether Vellum-owned `:::note`/`:::warning` should be the only callout path.
5. Dirty workspace policy: fail by default or warn by default. Current product direction suggests fail by default, with `--allow-dirty`.

---

## Non-Goals to Reaffirm Before Implementation

- No themes.
- No categories.
- No user templates.
- No dual Markdown renderer.
- No Typst.
- No inline code highlighting.
- No line highlighting.
- No line numbers.
- No default network access.
- No async runtime.
- No JS build pipeline.

---

## Execution Notes

Plan complete and saved to `vellum/docs/plans/2026-07-01-vellum-product-architecture.md`.

When implementation begins, use `superpowers:executing-plans` and execute task-by-task. Keep the MVP strict: if a feature pressures the design toward themes/plugins/render swaps, cut it or defer it.
