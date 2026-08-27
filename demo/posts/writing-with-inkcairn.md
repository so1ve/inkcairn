---
published: 2026-08-28
---

# Writing with Inkcairn

The part I like most about Inkcairn is deliberately unremarkable: the source is
plain Markdown, the metadata is small, and every post remains useful without the
generator that publishes it.

![A simple Inkcairn mark](/assets/inkcairn.svg)

## Start with the thought

I want **structure to follow the writing**, not the other way around. A draft can
begin as a paragraph, acquire *emphasis* where the argument needs it, lose a
~~bad sentence~~, and collect `inline code` or an
[ordinary reference](https://www.rust-lang.org/) without changing tools.

> A writing tool should get out of the way until the writing needs it.

- Capture the idea before polishing it.
- Keep source files readable in any editor.
- Add structure only when it helps the reader:
  1. Name the sections.
  2. Put supporting detail near the claim.

### Before publishing

- [x] State the central idea early
- [x] Remove the paragraph that merely repeats it
- [ ] Read the final draft away from the editor

## A small content model

| Part | Choice |
| --- | --- |
| Source | Markdown files |
| Title | First top-level heading |
| Description | Opening paragraphs |
| Dates | Git history, with an explicit override when needed |

## Details without interruption

> [!NOTE]
> A callout earns its visual weight only when missing it would change how the
> surrounding section is understood.

Footnotes are useful for provenance and side paths that should remain available
without taking over the main argument.[^notes]

[^notes]: The best footnote is still short enough that returning to the sentence is easy.

## Code should belong to the story

```rust
fn is_draft(path: &Path) -> bool {
    path.file_stem().unwrap().to_string_lossy().ends_with(".draft")
}
```

A diff is clearer than two nearly identical blocks when the change itself is the
point:

```rust,diff
-fn title(path: &Path) -> String {
-    humanize(path.file_stem().unwrap())
+fn title(path: &Path, heading: Option<&str>) -> String {
+    heading.unwrap_or_else(|| path.file_stem().unwrap().to_str().unwrap()).into()
}
```

## Why ordinary files?

<details>
  <summary>Because publishing software changes faster than writing does.</summary>
  <p>A directory of text and images is easy to inspect, move, version, and rebuild with a different tool years later.</p>
</details>
