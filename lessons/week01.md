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
which are a super neat way to merge unit testing and documentation. Most
importantly you get a nice and modern build system and package ecosystem,
instead of trying to futz about with make or cmake or visual studio settings or
anything else.

## How Much Should I Already Know?

I assume that you've programmed before, but not necessarily written any Rust
before.

If you're new to rust you should probably read through [The Rust Book
(2e)](https://doc.rust-lang.org/book/second-edition/index.html) at least once,
and in this very first lesson I'll be extra detailed about what's going on
because it's a lot to take in at once.

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

Some of those are probably not necessary. I don't know which. The full
documentation for the `Cargo.toml` file is [in the rust standard
reference](https://doc.rust-lang.org/cargo/reference/manifest.html).

Now we got to get a window open and draw a little `'@'`. We're using
[dwarf-term](https://crates.io/crates/dwarf-term) for drawing, which will let us
do Dwarf Fortress style graphics (backed by OpenGL 3.3). When approaching a new
lib, it's best to see if there are [any
examples](https://github.com/Lokathor/dwarf-term-rs/tree/master/examples). Oh,
look, there are. The wonders of doing a tutorial with a lib that you wrote
yourself at the last minute. Let's just copy that _entire thing_ into our
project. We'll save it in `src/bin/` as `kasidan.rs` (just a random name for the
game), and then `cargo` will automatically know to build that file into
`kasidan.exe` just based on the file's location. So, let's type `cargo run` and
give it a go.

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
     Running `target\debug\kasidan.exe`
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
use `HashSet` right away, but we'll end up with a `HashMap` too soon enough I'm
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
anything. Someday.

```rust
fn main() {
  unsafe {
    let mut term = DwarfTerm::new(TILE_GRID_WIDTH, TILE_GRID_HEIGHT, "Kasidan Test").expect("WHOOPS!");
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
match on the event.

```rust
        Event::WindowEvent { event: win_event, .. } => match win_event {
```

If we have a window event we'll match on the inner window event.

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
