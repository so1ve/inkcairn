---
published: 2026-08-24
---

# Types and text

Text is rarely “just a string.” The moment a program needs to compare, display,
slice, normalize, or persist it, an implicit model of text becomes part of the
design.

## A small observation

Good representations make the common operation obvious. A path is not merely
text, a user-facing label is not an identifier, and bytes received from the
outside world do not become valid Unicode because a convenient API expects a
`String`.

Types are most valuable here when they preserve a distinction the program would
otherwise have to remember everywhere.
