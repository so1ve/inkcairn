# Reading old code slowly

Old code is easiest to misunderstand when approached as a cleanup exercise.
Before changing it, I try to learn which constraints shaped it and which of
those constraints still exist.

## Begin at the boundary

The public entry point usually tells a clearer story than the helpers beneath
it. Follow one ordinary input through the program before collecting suspicious
functions or unusual branches.

## Keep an evidence list

I write down three kinds of observations:

- behavior that callers visibly depend on;
- complexity caused by a requirement that still exists;
- complexity whose original requirement can no longer be found.

The third list is where simplification begins, but only after the first two are
understood.
