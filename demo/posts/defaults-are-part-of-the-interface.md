# Defaults are part of the interface

A default is a decision made on behalf of every person who does not stop to
configure the software. That makes it more important than most advanced options,
even when it occupies only one line of code.

## The common path

Good defaults make the common path obvious and safe. They should express what
the program believes most people mean, while leaving an honest route to the less
common cases.

The difficult part is resisting defaults that merely hide uncertainty. If the
program cannot infer an important choice, asking once is kinder than guessing
forever.

## Configuration has a cost

Every option adds documentation, combinations, and states that future changes
must preserve. Before adding one, it is worth asking whether the underlying
behavior is actually stable enough to deserve a permanent switch.
