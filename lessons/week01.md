# Week 01

# Part 0: Introduction Stuff

## Prior Art

It should first be noted that there is [another "roguelike in rust"
tutorial](https://tomassedovic.github.io/roguelike-tutorial/), it was made as
part of the 2017 summer roguelike event.

**Why Not Use That One?** Fair question. A lot of that tutorial is kinda OO-ish
in mindset, because it's like _really_ close to the python version in a lot of
ways. I don't like that. Also, that tutorial relies on the rust TCOD library,
which is dumb, because TCOD itself relies on SDL, which is dumb, because we have
pure-rust options at our disposal. __So we won't be using the TCOD library!__
Not even a little.

## Why Rust?

If you ever talk about programming in Rust you are legally required to repeat
the following meme:

> Rust is a systems language pursuing the trifecta: safety, concurrency, and speed.

There, I did it.

So, really, why Rust? Because it's like C but without the crufty nonsense.
(Instead, you get a totally different pile of nonsense knocking at your door.)
However, you _also_ get a nifty type system that's familiar to Haskell
programmers (and ML programmers in general, I'm told). You also get doc tests,
which are a super neat way to merge unit testing and documentation (your
examples in the docs don't go out of date). Most importantly you get a nice and
modern build system and package ecosystem, instead of trying to futz about with
make or cmake or visual studio project settings or anything else like that.

## How Much Should I Already Know?

If you _don't_ want to actually do any of this yourself then you should be fine
without any previous rust experience at all. Feel free to just glance through at
how things kinda work if you don't want to start in on a whole new programming
language.

If you _do_ want to be able to do this sort of stuff yourself, you'll have to
have already read [The Rust Book (2nd
Edition)](https://doc.rust-lang.org/book/second-edition/index.html) (don't
worry, it's free), and you'll probably want to have already written some simple
rust programs of your own.

Please note that the tutorial _will_ attempt to target the harder sorts of
material where possible. There's little point in a tutorial that only shows you
how to do the easy things, you could have done those yourself.

## Installing The Stuff

To install rust you'll wanna use the [rustup](https://rustup.rs/) tool. If
you're on windows and you want to use the MSVC toolchain you'll also need the
Visual Studio C++ tools, but rustup will explain all that and tell you where to
go and such when you run it.

The tutorial currently requires the _nightly_ branch of rust, but as far as I
know we can move to stable once 1.27 is released.

# Part 1: Drawing the '@' symbol and moving it around

So we get our new project going. You can use `cargo new`, but honestly I'm dumb
and I just copy files from a previous project into a new folder and then edit
them to be what I need. To get our project off the ground we'll need a file
called `Cargo.toml`. We'll be drawing with a lib called `dwarf-term`, so we'll
add that into the dependency section right away.

```toml
[package]
name = "roguelike-tutorial-2018"
version = "0.1.0-pre"
authors = ["Lokathor <zefria@gmail.com>"]
repository = "https://github.com/Lokathor/roguelike-tutorial-2018"
readme = "README.md"
keywords = ["roguelike","tutorial"]
description = "The Summer Roguelike Tutorial (2018 edition)."
license = "0BSD"
publish = false

[dependencies]
dwarf-term = "0.1"
```

Some of those are probably not necessary. I don't honestly know which. The full
documentation for the `Cargo.toml` file is [in the rust standard
reference](https://doc.rust-lang.org/cargo/reference/manifest.html).

Now we got to get a window open and draw a little `'@'`. We're using
[dwarf-term](https://crates.io/crates/dwarf-term) for drawing, which will let us
do Dwarf Fortress style graphics (backed by OpenGL 3.3). When approaching a new
lib, it's best to see if there are [any
examples](https://github.com/Lokathor/dwarf-term-rs/tree/master/examples). Oh,
look, there are. The wonders of doing a tutorial with a lib that you wrote
yourself at the last minute. Let's just copy that _entire thing_ into our
project. We'll save it in `src/bin/` as `kasidin.rs`. Kasidin is just a random
name for the game, it could be any file name really. Once it's in the `src/bin/`
directory, `cargo` will automatically know to build that file into a binary just
based on the file's location. So, let's type `cargo run` and give it a go.

```
D:\dev\roguelike-tutorial-2018>cargo run
   Compiling khronos_api v2.2.0
   Compiling cfg-if v0.1.3
   Compiling bitflags v1.0.3
   Compiling winapi v0.3.5
   Compiling lazy_static v1.0.1
   Compiling libc v0.2.42
   Compiling retro-pixel v0.3.0
   Compiling log v0.4.2
   Compiling xml-rs v0.7.0
   Compiling shared_library v0.1.8
   Compiling gl_generator v0.9.0
   Compiling glutin v0.15.0
   Compiling gl v0.10.0
   Compiling winit v0.13.1
   Compiling dwarf-term v0.1.0
   Compiling roguelike-tutorial-2018 v0.1.0-pre (file:///D:/dev/roguelike-tutorial-2018)
    Finished dev [unoptimized + debuginfo] target(s) in 10.75s
     Running `target\debug\kasidin.exe`
```

Wewe! That's a lot of dependencies friend! The beauty of `cargo` is that we
didn't have to go and get any of it ourselves. And it opens a window! There's a
greenish-yellow `'@'`, and the arrow keys move it around. Neat. Now let's review
all the stuff we just put in.

```rust
//! The main program!

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(dead_code)]
```

The first part is a _doc comment_. Normally they start with `///` on each line
and then apply to the item below them, but `//!` means "document the outer thing
instead". This is the normal way to document a module. We don't have much to say
here yet, but we might have stuff to say about other modules later on.

Next we have a few _attributes_, which are normally like `#[thing]` and then
document something below them, but again with the `!` in there they instead
apply an attribute to the outer thing, in this case the module they're in. Since
this module is also the root of the entire program, they apply to our program as
a whole.

* The first is a windows specific attribute so that when we use release mode
  it'll compile our program as a windows mode program (as opposed to a console
  mode program). On non-windows this will just quietly have no effect.
* The rest are a bunch of directives to shut up compiler warnings. They're not
  all good in "production" code, but for getting something on screen they're
  fine.

```rust
extern crate dwarf_term;
pub use dwarf_term::*;

// std
use std::collections::{HashMap, HashSet};
```

These lines say that we'll use the `dwarf_term` crate, and then pull in
everything from the `dwarf_term` module which is the root of that crate. The
`std` crate doesn't need to be pulled in with "extern crate" in a normal
program, but I put a little comment line just so that when we have several
crates and each of their use statements, it all matches up. From the standard
library we'll want to use the `HashMap` and `HashSet` types. Actually we'll only
use `HashSet` right away, but we'll end up with a `HashMap` soon enough I'm
sure.

```rust
const TILE_GRID_WIDTH: usize = 66;
const TILE_GRID_HEIGHT: usize = 50;
```

The tiles for dwarf_term are each 12x12 pixels, and so with this many grid cells
we get a window that's around 800x600, which is a comfortable size on a large
screen and still doesn't go off screen even on an older monitor. Like I said
earlier, `dwarf_term` was put out at the last minute to be available for this
tutorial series, so the 0.1 version doesn't have configurable tilesets or
anything like that. Someday I'll get around to that.

```rust
fn main() {
  unsafe {
    let mut term = DwarfTerm::new(TILE_GRID_WIDTH, TILE_GRID_HEIGHT, "Kasidin Test").expect("WHOOPS!");
    term.set_all_foregrounds(rgb32!(128, 255, 20));
    term.set_all_backgrounds(0);
```

This declares our main, marks the whole thing as using `unsafe` code (necessary
to open the window and to redraw the window, since those do a lot of FFI calls),
makes our terminal, and then sets the foreground and background color for all
locations. Creating the terminal can fail, and if it does we just print "whoops"
and panic, since there's not much else to do right now. The only time we fail to
open a window is if there's already a window, so since we know there's not
already a window, we'll be fine.

Also, I'm using 2 spaces per tab level. The "rust standard" is the more usual
4-spaces, but I just use 2 because I usually seem to end up with a lot of brace
levels.

```rust
    // Main loop
    let mut running = true;
    let mut keys_new = HashSet::new();
    let mut keys_held = HashSet::new();
    let mut watcher_position: (isize, isize) = (5, 5);
    while running {
```

We don't have much game setup to do, so we just throw out a few variables and go
into our core loop.

```rust
      term.poll_events(|event| match event {
```

First thing we do at the start of each frame is poll for any pending events and
match on the event. The `dwarf-term` crate actually doesn't handle events much
at all, it just passes along your events poll closure to the actual events
polling that [winit](https://crates.io/crates/winit) does (the window lib that
`dwarf-term` is built on top of). So, the event handling you'll see here is the
same as you'd use with any other `winit` program.

```rust
        Event::WindowEvent { event: win_event, .. } => match win_event {
```

An event can be a WindowEvent or a DeviceEvent, so if we have a window event
we'll match on that inner window event.

```rust
          WindowEvent::CloseRequested
          | WindowEvent::KeyboardInput {
            input:
              KeyboardInput {
                state: ElementState::Pressed,
                virtual_keycode: Some(VirtualKeyCode::Escape),
                ..
              },
            ..
          } => {
            running = false;
          }
```

If we get a CloseRequested (probably because the user clicked the 'X' in the
corner or something), or if the user presses Esc, we'll stop running. Because
the match for the poll happens in another function, we can't use `break`
directly, so setting `running` to false is a bit of a work around.

```rust
          WindowEvent::KeyboardInput {
            input:
              KeyboardInput {
                state: ElementState::Pressed,
                virtual_keycode: Some(key),
                ..
              },
            ..
          } => {
            keys_new.insert(key);
          }
```

If there's any other key press (besides Esc that is), we record it as new this
frame. We'll handle moving the new keys into the held keys later on.

```rust
          WindowEvent::KeyboardInput {
            input:
              KeyboardInput {
                state: ElementState::Released,
                virtual_keycode: Some(key),
                ..
              },
            ..
          } => {
            keys_held.remove(&key);
          }
```

If a key is being released, delete it from our held keys. In some situations
it's possible to get key released events for keys we didn't even think were
pressed (because of keyboard rollover), but `HashSet` is okay with being asked
to delete something that's not there.

```rust
          _ => {}
        },
        _ => {}
      });
```

For any other type of window event, or any other type of event at all, we do
nothing. Later on we might support the mouse or something, but not now.

```rust
      for key in keys_new.drain() {
        keys_held.insert(key);
        match key {
          VirtualKeyCode::Up => watcher_position.1 += 1,
          VirtualKeyCode::Down => watcher_position.1 -= 1,
          VirtualKeyCode::Left => watcher_position.0 -= 1,
          VirtualKeyCode::Right => watcher_position.0 += 1,
          _ => {}
        }
      }
```

New keys this frame we go over, moving them to the held keys, and then updating
our position if it's an arrow key. If one or more keys are held over time, the
most recently pressed key will generate more key pressed events. So if you press
A and then D and hold both, you'll get one key pressed for A, and then a lot of
key pressed for D. I don't really know what we wanna do for input, so we'll just
go along with this for now. We'll probably want to cut it down later, since a
roguelike usually just responds to single "typed" key sorts of events (press i,
press a, press enter, etc)

```rust
      term.set_all_ids(b' ');

      term
        .get_id_mut((watcher_position.0 as usize, watcher_position.1 as usize))
        .map(|mut_ref| *mut_ref = b'@');
```

We clear the tile id of all the locations, and then "draw" the main character to
their position if it's within the screen. Later on we'll have walls to keep the
player in bounds, but right now they can walk right off the edge. If they do,
it'll just stop drawing them.

```rust
      term
        .clear_draw_swap()
        .map_err(|err_vec| {
          for e in err_vec {
            eprintln!("clear_draw_swap error: {:?}", e);
          }
        })
        .ok();

      // Error check
    }
  }
}
```

Finally, we push out all our changes to the screen at the end of the time. The
`dwarf_term` lib always uses vsync, so this will also end up blocking as
necessary to prevent us from going faster than the monitor refresh rate. Right
now we don't do any real time animation, but we could, and this would keep us at
a steady cap of 60fps.

## So what's wrong so far?

Well, first of all, there's a comment that says "error check" but we don't
actually check any errors. That's just some copy paste junk that's snuck in
there. I'll go fix the example at some point as well I'm sure.

Next, we're using `unsafe` too much. We really should try to limit it down when
we can. The only times we actually need it are for making a new window (which
does a bunch of GL calls) and re-drawing and flipping the window (which also
does a different bunch of GL calls). The rest of the time it's all fully safe
code. That's an easy fix too.

Also, our input reading isn't the best system right now. Roguelike games are
usually "curses-ish" with their input even if they're not using curses, so we're
not likely to care about what keys are held over time, just which new ones were
pressed. However, because the new keys pressed on each frame go into a set
before they get processed by the game, they can theoretically end up getting
processed out of order. I think we'll want to keep that in mind, and maybe
switch to a Vec in the next lesson. Right now, "up then right" is the same as
"right and then up", but once there's walls to bump into we'll care. That part
we'll fix next lesson though.
