---
published: 2024-04-21
---

# What a type name promises

A type name is a small promise about the values a program may contain. When the
name and the representation disagree, every caller must quietly repair that
promise in its own head.

## Prefer the domain word

`String` describes storage. `Slug`, `RepositoryUrl`, and `PublishedDate`
describe roles. A newtype is worthwhile when the role carries an invariant or
prevents two identical representations from being confused.

It is less useful when it only moves getters and constructors around without
removing decisions from callers.

## Read the signatures together

One function signature can look reasonable in isolation. A module's full set of
signatures reveals whether its vocabulary is coherent or whether the same idea
appears under several names.
