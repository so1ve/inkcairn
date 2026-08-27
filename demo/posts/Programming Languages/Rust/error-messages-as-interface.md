# Error messages are part of the interface

An error variant may be designed for a program, but its final message is read by
a person who needs to decide what to do next. That makes wording and context part
of the interface rather than decoration added at the end.

## Name the failed operation

“Invalid input” says almost nothing. “Cannot read `notes/index.md`” identifies
both the attempted operation and its target. The underlying operating-system
error can then explain why it failed.

## Context should earn its place

Repeatedly wrapping the same error makes a message longer without making it more
useful. Add context at the boundary that knows the concrete operation; let clear
errors pass through intermediate layers unchanged.

```rust
let source = fs::read_to_string(path)
    .with_context(|| format!("cannot read {}", path.display()))?;
```
