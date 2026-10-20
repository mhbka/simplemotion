## simplemotion
a simple library that takes the [decompiled](https://github.com/UlyssesWu/FreeMote) output of an `.mtn` animation file and generates `AnimationCommands` at a given frame/time, which contain all the info you need to animate in a 2D canvas of your choice.

### note
it does NOT support mesh deformations, just simple affine transforms for 2D stuff.

### try
```
cargo run --release --example inspector -- -m anim\
cargo run --release --example pendulum
```