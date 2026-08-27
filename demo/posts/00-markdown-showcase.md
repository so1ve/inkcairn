# A small Markdown showcase

## Heading 2

### Heading 3

#### Heading 4

##### Heading 5

###### Heading 6

## Text

A paragraph can include **strong emphasis**, *gentle emphasis*,
***both at once***, or a ~~thought that did not survive editing~~. Names such as
`inkcairn.md` belong comfortably inline, while an
[ordinary link](https://commonmark.org/) and an automatic link such as
<https://www.rust-lang.org/> remain easy to spot in the source.

---

## Lists and small plans

An unordered list works well for a collection whose order does not matter:

- Plain text files
- A predictable directory layout
- A build that leaves the source untouched
  - Posts stay under `posts/`
  - Images stay under `assets/`

Use an ordered list when sequence does matter:

1. Write the first draft.
2. Read it away from the editor.
3. Publish only when it says what you meant.

A task list can hold the last few details:

- [x] Choose a title
- [x] Add a useful example
- [ ] Stop polishing and publish

## Quotations and callouts

> Good tools leave more room for the work itself.
>
> A blockquote can hold more than one paragraph, but it should still earn the
> interruption.

> [!TIP]
> Keep the source simple enough to understand in any text editor. The rendered
> page can take care of the presentation.

## Tables

Tables are handy for compact comparisons rather than long prose:

| Source | Result | Kept in Git |
| :--- | :---: | ---: |
| Markdown | Article | Yes |
| Image | Static asset | Yes |
| Generated HTML | Published site | No |

## Code and diffs

Fenced blocks keep code readable and add syntax highlighting to the page:

```typescript
type Post = {
  title: string;
  published: Date;
};

const describe = (post: Post) => `${post.title} · ${post.published.getFullYear()}`;
```

When the change matters more than the surrounding file, a diff says it directly:

```diff
-const status = "draft";
+const status = "published";
```

## Images and hidden details

Images use the same familiar syntax as links:

![The Inkcairn mark](/assets/inkcairn.svg)

<details>
  <summary>Some details can wait until they are wanted.</summary>
  <p>This small HTML element is useful for an aside that would otherwise interrupt the article.</p>
</details>

## Footnotes

A footnote keeps a useful aside nearby without forcing it into the sentence.[^1]
The reader can follow it and return to exactly the same place.

[^1]: Footnotes can contain links, emphasis, and other ordinary Markdown.
