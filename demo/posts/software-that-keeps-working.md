---
published: 2023-09-14
---

# Software that keeps working

Longevity rarely comes from predicting every future requirement. It comes from
choosing inputs and outputs that remain understandable when the implementation
between them has to change.

## Durable edges

Plain text, documented URLs, and standard formats are not exciting choices, but
they give future software something stable to meet. The internals can then be
rewritten without turning every old document into a migration project.

## Fewer hidden obligations

Persistence creates promises. A cache can be deleted; an authored document
cannot. Treating those two kinds of data differently keeps defensive machinery
focused where losing information would actually matter.
