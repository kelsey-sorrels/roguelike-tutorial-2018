# Week 02

# Part 02: A Static Dungeon

So, the `'@'` moves anywhere we like. He is unconstrained, but by being able to
do anything, he also has nothing to do. Let's give him some limits.

First we will make a static dungeon shape, and then in the next part will we
make random dungeon shapes. Also, we will make a dungeon that is _bigger_ than
our screen's display area, so that we are forced to develop a concept of a
scrolling view of the dungeon.

At the behest of a friendly imp we'll be doing this whole game using an [Entity Component System](https://en.wikipedia.org/wiki/Entity%E2%80%93component%E2%80%93system). In Rust that means that we want to be using a lib called [specs](https://crates.io/crates/specs/). Just add that to our `Cargo.toml` file's dependency section...

```toml
[dependencies]
dwarf-term = "0.1"
specs = "0.11"
```

And give that a `cargo build` so that `cargo` fetches it for us...

```
D:\dev\roguelike-tutorial-2018>cargo build
    Updating registry `https://github.com/rust-lang/crates.io-index`
 Downloading specs v0.11.2
 Downloading mopa v0.2.2
 Downloading hibitset v0.5.0
 Downloading derivative v1.0.0
 Downloading shred v0.7.0
 Downloading tuple_utils v0.2.0
 Downloading crossbeam v0.3.2
 Downloading shrev v1.0.1
 Downloading fnv v1.0.6
 Downloading shred-derive v0.5.0
 Downloading atom v0.3.5
 Downloading syn v0.10.8
 Downloading quote v0.3.15
 Downloading itertools v0.5.10
 Downloading unicode-xid v0.0.4
 Downloading fxhash v0.2.1
 Downloading smallvec v0.6.2
 Downloading parking_lot v0.5.5
 Downloading owning_ref v0.3.3
 Downloading parking_lot_core v0.2.14
 Downloading stable_deref_trait v1.1.0
 Downloading syn v0.11.11
 Downloading synom v0.11.3
   Compiling winapi v0.3.5
   Compiling nodrop v0.1.12
   Compiling memoffset v0.2.1
   Compiling scopeguard v0.3.3
   Compiling rayon-core v1.4.0
   Compiling smallvec v0.6.2
   Compiling stable_deref_trait v1.1.0
   Compiling either v1.5.0
   Compiling quote v0.3.15
   Compiling unicode-xid v0.0.4
   Compiling byteorder v1.2.3
   Compiling atom v0.3.5
   Compiling mopa v0.2.2
   Compiling fnv v1.0.6
   Compiling crossbeam v0.3.2
   Compiling tuple_utils v0.2.0
   Compiling crossbeam-utils v0.2.2
   Compiling num_cpus v1.8.0
   Compiling arrayvec v0.4.7
   Compiling owning_ref v0.3.3
   Compiling synom v0.11.3
   Compiling itertools v0.5.10
   Compiling fxhash v0.2.1
   Compiling syn v0.10.8
   Compiling syn v0.11.11
   Compiling crossbeam-epoch v0.3.1
   Compiling crossbeam-deque v0.2.0
   Compiling shred-derive v0.5.0
   Compiling derivative v1.0.0
   Compiling rand v0.4.2
   Compiling winit v0.13.1
   Compiling glutin v0.15.0
   Compiling parking_lot_core v0.2.14
   Compiling parking_lot v0.5.5
   Compiling rayon v1.0.1
   Compiling dwarf-term v0.1.0
   Compiling shrev v1.0.1
   Compiling hibitset v0.5.0
   Compiling shred v0.7.0
   Compiling specs v0.11.2
   Compiling roguelike-tutorial-2018 v0.1.0-pre (file:///D:/dev/roguelike-tutorial-2018)
    Finished dev [unoptimized + debuginfo] target(s) in 28.11s
```

Wow, that's kinda a lot. You might note that it even rebuilt some of the crates
we already had built, now that it's building more things together at once. Heck,
since we're already editing the `Cargo.toml` file we might as well turn on
Link-time Optimization for the `release` and `bench` profiles. Also, we'll turn
on `debug-assertions` for the `release` profile for now, just to make sure we're
not hitting anything weird. We can turn that back off later of course.

```toml
[profile.release]
lto = true
debug-assertions = true

[profile.bench]
lto = true
```

Now, `cargo` won't know on its own to delete the old build stuff we won't be
using any more now that we've got these new settings in place, so we'll throw
out a quick `cargo clean && cargo build`,

```
   [all sorts of package versions cut for space]
   Compiling roguelike-tutorial-2018 v0.1.0-pre (file:///D:/dev/roguelike-tutorial-2018)
    Finished dev [unoptimized + debuginfo] target(s) in 16.57s
```

TODO: make a static map

# Part 03: A Random Dungeon

TODO: make a random map
