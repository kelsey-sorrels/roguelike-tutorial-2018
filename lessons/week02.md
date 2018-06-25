# Week 02

FIRST THINGS FIRST. If you're doing the demo from last time, please use `cargo
update`. I've put out a `0.1.1` version of `dwarf_term`, which fixes two
different bugs that appeared when people started trying things out.

# Part 02: A Static Dungeon

So, the `'@'` moves anywhere we like. He is unconstrained, but by being able to
do anything, he also has nothing to do. Let's give him some limits.

First we will make a static dungeon shape, and then in the next part will we
make random dungeon shapes.

Let's start putting some data types to this. First we'll adjust our std imports.

```rust
use std::collections::hash_map::*;
use std::ops::*;
```

The stuff in `std::ops` is mostly for operator overloading.

```rust
#[derive(Debug, Clone, Default)]
struct GameWorld {
  player_location: Location,
  creatures: HashMap<Location, Creature>,
  terrain: HashMap<Location, Terrain>,
}
```

A `GameWorld` has a player_location, some creatures, and some terrain. Seems
good enough for an `'@'` and at least one wall tile.

```rust
#[derive(Debug, Clone, Copy)]
struct Creature {}

#[derive(Debug, Clone, Copy)]
enum Terrain {
  Wall,
  Floor,
}

impl Default for Terrain {
  fn default() -> Self {
    Terrain::Wall
  }
}
```

We don't actually have anything to say about what makes up a `Creature`, so we
say nothing at all. The `Terrain` is either a Wall or Floor, and we'll even say
that the default Terrain is a Wall tile. A lot of stuff in rust uses the
`Default` trait, so you should try to implement it as often as it makes sense
for a type. Unfortunately, you can't derive the default of an enum, so you have
to write it out.

```rust
#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, Hash)]
struct Location {
  x: i32,
  y: i32,
}

impl Location {
  fn as_usize(self) -> (usize, usize) {
    (self.x as usize, self.y as usize)
  }
}

impl Add for Location {
  type Output = Self;
  fn add(self, other: Self) -> Self {
    Location {
      x: self.x + other.x,
      y: self.y + other.y,
    }
  }
}
```

A `Location`, for now, is just a 2d point. We'll add z later on. The `Add` trait
lets us do `a + b` with the Location type.

Now we just change it so that when the player pressed an arrow key we tell the
game to move the player. That old input system was not good, so we'll just
listen for key presses for now, ignoring if a key is being held down during
other keys or not.

```rust
  // Main loop
  let mut running = true;
  let mut pending_keys = vec![];
  'game: loop {
```

And once we've got all out inputs gathered we'll dispatch on them one at a time.

```rust
    for key in pending_keys.drain(..) {
      match key {
        VirtualKeyCode::Up => game.move_player(Location { x: 0, y: 1 }),
        VirtualKeyCode::Down => game.move_player(Location { x: 0, y: -1 }),
        VirtualKeyCode::Left => game.move_player(Location { x: -1, y: 0 }),
        VirtualKeyCode::Right => game.move_player(Location { x: 1, y: 0 }),
        _ => {}
      }
    }
```

Okay, so we're kinda pushing off the real work until later... Let's keep going
with stuff in the main loop though.

Once we've got all the keys processed we need to draw what the world looks like
now.

```rust
    {
      let (mut fgs, mut bgs, mut ids) = term.layer_slices_mut();
      for (scr_x, scr_y, id_mut) in ids.iter_mut() {
        let loc_for_this_screen_position = Location {
          x: scr_x as i32,
          y: scr_y as i32,
        };
        match game.creatures.get(&loc_for_this_screen_position) {
          Some(ref creature) => {
            *id_mut = b'@';
            fgs[(scr_x, scr_y)] = rgb32!(255, 255, 255);
          }
          None => match game.terrain.get(&loc_for_this_screen_position) {
            Some(Terrain::Wall) => {
              *id_mut = WALL_TILE;
              fgs[(scr_x, scr_y)] = rgb32!(155, 75, 0);
            }
            Some(Terrain::Floor) => {
              *id_mut = b'.';
              fgs[(scr_x, scr_y)] = rgb32!(128, 128, 128);
            }
            None => {
              *id_mut = b' ';
            }
          },
        }
      }
    }
```

So to start, we grab all the layers of the `DwarfTerm`. It's in three layers
that each hold one value instead of one layer that holds three values per cell
because that's how the textures get uploaded to opengl. They're all the same
size, so if we iterate one we can just use the same position to write to the
other two as well. For each camera location, we convert that into world
coordinates, then check if there's a creature there, and if there is we draw
that, otherwise we draw the terrain there. If there's neither there we just
clear the cell (this way our camera doesn't have to be locked at the edges of
the map and can drift into the void safely). We'll have to adjust this once we
do field of view things, but for now the player will have perfect knowledge of
the game world.

Now we have to go back and _actually fill in_ our movement code.

```rust
impl GameWorld {
  fn move_player(&mut self, delta: Location) {
    let player_move_target = self.player_location + delta;
    if self.creatures.contains_key(&player_move_target) {
      // LATER: combat will go here
    } else {
      match *self.terrain.entry(player_move_target).or_insert(Terrain::Floor) {
        Terrain::Wall => {
          // Accidentally bumping a wall doesn't consume a turn.
          return;
        }
        Terrain::Floor => {
          let player = self
            .creatures
            .remove(&self.player_location)
            .expect("The player wasn't where they should be!");
          let old_creature = self.creatures.insert(player_move_target, player);
          debug_assert!(old_creature.is_none());
          self.player_location = player_move_target;
        }
      }
    }
    // LATER: other creatures act now that the player is resolved.
  }
}
```

That's... a lot. Also, we're doing some very rust specific jiggery hackery.

So to move the player, we need the game world and a location delta to move by.
Any sort of rule that the player can only move 1 tile per step? That's in the
hands of the caller as far as this code is concerned. We'll add the player's
current location and the delta to compute where they're trying to move to.

First, we check if there is a creature there. If there is one, we would do an
attack, but since we don't have other creatures or combat rules yet we'll just
leave a note.

Then, we look up what terrain is at the location that the player is trying to
step to. Since we've only got a HashMap for terrain, and since we're only going
to put one map tile into it, we'll look up the entry and then the
`.or_insert(Terrain::Floor)` part will make there be a floor entry there if we
didn't find anything at all. So now we know we're _always_ gonna have Wall or
Floor, so we match on that.

If there's a wall, you can't go there. We return without taking up a turn. If
there's a floor, we take the player out of where they are, and put them into the
new location. We also update the player's current location in the game data. Why
are we storing the player's location information in two places at once? Isn't
that error prone? Yes, it is, but it also makes it easy to check the player's
location, which we're gonna be doing a lot I think. We can take that out later
if we need to.

Finally, now that the player has performed a turn, we'd give a turn to all the
other creatures. However, we have no other creatures, so that's it. Now we just
have to populate our world.

```rust
  let mut game = GameWorld {
    player_location: Location { x: 5, y: 5 },
    creatures: HashMap::new(),
    terrain: HashMap::new(),
  };
  game.creatures.insert(Location { x: 5, y: 5 }, Creature {});
  game.terrain.insert(Location { x: 10, y: 10 }, Terrain::Wall);
```

Now you can walk around, and bump into the wall. Because the game adds a floor
any place that you step that doesn't have terrain already, you can always walk
into the void, and you leave a little trail of floor tiles behind you.

[here](https://github.com/Lokathor/roguelike-tutorial-2018/tree/4721437ff289a3b71c3f8b98b13a3736f924c17a)
is the link to the exact state of the project right now, in case I forgot to
explain anything or didn't explain anything well enough. Give it a try.

# Part 03: A Random Dungeon

Having a single wall tile is an okay demo, but roguelikes need random maps!
Before we can have a whole random map, we need some random anything to build
upon. The normal crate for randomization is the
<tt>[rand](https://crates.io/crates/rand)</tt> crate, but it's the kind of crate
that's designed to cover _all possible_ use cases, which (to me) makes it
sometimes hard to understand how to use for our own situation. Because of that,
I made my own minimal library that handles randomization things for roguelike
sorts of purposes quite nicely called
<tt>[randomize](https://crates.io/crates/randomize)</tt>.

We won't be using either of those.

This is a tutorial, so we're going to teach and learn a bit. It's easy to grab a
library and just throw out some code, but to me a big part of this project of
"do a roguelike without TCOD" is to hopefully remind people that that _the TCOD
library does nothing you can't do yourself_. Now, I admit that I didn't do that
with the `DwarfTerm` stuff in Lesson 01. I do encourage you to go [read the
source](https://github.com/Lokathor/dwarf-term-rs) for how we're drawing to the
screen. However, the problem with explaining what `DwarfTerm` does is that if
you _do_ already know how OpenGL works then you'll understand the entire crate
by reading a [single shader
file](https://github.com/Lokathor/dwarf-term-rs/blob/master/src/dwarf.frag), and
if you _don't_ know how OpenGL works then you'd first need to go [read about
eight lessons of background](https://learnopengl.com/Getting-started/OpenGL) to
get you up to speed on that, and then you can read the shader file. At some
point you just have to decide what's considered a "fundamental" element and move
on.

So, we'll build our own RNG right "from scratch" even though we didn't build our
own graphics lib or cross-platform windowing lib. It's an arbitrary decision,
but we'll stick to it. In a "real" project you might be well served just picking
up a lib off the shelf, but for now we're going to learn when we can.

## Part 03a: The "Writing our own RNG" Tangent

So what's a random number generator (RNG)? Well first of all they're not
actually random, not the ones we're using. They're just hard to predict if you
don't know the internal state, but it's still totally deterministic like the
rest of a computer, so we're more technically using pesudo-random number
generators (PRNG). They've got some sort of state, and then they do some math,
and then they change their internal state and then decide on an output.

```rust
fn very_bad_prng(state: &mut u32) -> u32 {
  *state += 1;
  state
}
```

Okay, so that's easy to do. Except our generator is too predictable because our
algorithm is dumb. So now we just need to know _what algorithm_ to use.

People have been doing computing for quite a while now, so as you might guess
there's several to pick from. Let's start with the [Linear Congruential
Generator](https://en.wikipedia.org/wiki/Linear_congruential_generator). It's
relatively easy to understand:

```
x[n+1] = (a * x[n] + c) % m

where

0 < m
0 < a < m
0 <= c < m
0 <= x[0] < m
```

Okay so we can get a less bad prng

```rust
fn lcg1(state: &mut u32) -> u32 {
  let a = 1;
  let c = 1;
  // we get "m = 2^32" for free by using wrapping_mul and wrapping_add
  *state = state.wrapping_mul(a).wrapping_add(c);
  state
}
```

Alright, we're cooking with gas. We've done what it said so we're all fine right?

> While LCGs are capable of producing pseudorandom numbers which can pass formal tests for randomness, this is extremely sensitive to the choice of the parameters m and a.[clarification needed] For example, a = 1 and c = 1 produces a simple modulo-m counter, which has a long period, but is obviously non-random.
> -- The Wikipedia Article

Whoops, gotta read that fine print I guess. So depending on our modulus value,
we can also pick `a` and `c` values to get a good generator. We'll stick with
modulus 32 because the CPU is good at doing that. So we look at the chart and
then pick one.

```rust
fn lcg2(state: &mut u32) -> u32 {
  // MS trusts these values, so it's safe right?
  let a = 214013;
  let c = 2531011;
  *state = state.wrapping_mul(a).wrapping_add(c);
  state
}
```

> As shown above, LCGs do not always use all of the bits in the values they produce.

Oh my.

```rust
fn lcg3(state: &mut u32) -> u32 {
  // OpenVMS trusts these values, so it's safe right?
  let a = 69069;
  let c = 1;
  *state = state.wrapping_mul(a).wrapping_add(c);
  state
}
```

So now we're safe, with a top quality generator, right?

> ## LCG derivatives

Oh, dear... hmm, seems like we have a few ways to make big LCGs out of small
LCGs, which is useful I guess. There's also something called a "permuted
congruential generator". It "applies an output transformation to improve its
statistical properties". Let's go there.

> It achieves excellent statistical performance with small and fast code, and
small state size.
> 
> A PCG differs from a classical linear congruential generator in three ways:
> 
> * the LCG modulus and state is larger, usually twice the size of the desired output,
> * it uses a power-of-2 modulus, which results in a particularly efficient
>   implementation with a full period generator and unbiased output bits, and
> * the state is not output directly, but rather the most significant bits of the
>   state are used to select a bitwise rotation or shift which is applied to the
>   state to produce the output.
>
> It is the variable rotation which eliminates the problem of a short period in
> the low-order bits that power-of-2 LCGs suffer from.

So we'll need `u64` instead of `u32`. Easy to do. We've already got a mod
power-of-2 going on, and double the size would stay as mod power-of-2 of course.
Then we add some bit math into our generator and we'll finally be all good
despite having a power of 2 modulus.

> The PCG family includes a number of variants.

Oh, gosh.

> These are combined into the following recommended output transformations,
> illustrated here in their most common sizes:

Ah, perfect. We'll use one of these.

```
count = (int)(x >> 59); x ^= x >> 18; return rotr32((uint32_t)(x >> 27), count);
```

So we

1. run a 64 bit version of the LCG.
2. use the highest 5 bit positions to pick a rotation amount. These are the best
   quality bits, so they give the best quality randomization of the rotation
   used in step 4. We'll talk about where the 5 comes from during step 4.
3. XOR our LCG output with itself shifted down 18 bits.
   * Where is the magic number 18 from? Let's check the article: "The constant
     is chosen to be half of the bits not discarded by the next operation
     (rounded down)". So, 64 - 27 = 37, int(37/2) = 18.
4. Perform a 32-bit right rotation of the LCG output shifted down by 27 bits.
   * The amount of the rotation is the number we got in step 2.
   * Again, 27 seems kinda magical, so why? "Given a 2<sup>b</sup>-bit input
     word, the top _b−1_ bits are used for the rotate amount, the
     next-most-significant 2<sup>b-1</sup> bits are rotated right and used as
     the output, and the low 2<sup>b-1</sup>+1−b bits are discarded." Oof let's
     unpack that carefully:
     * "Given a 2^b input word", Our input is 64 bit, so _b_ is 6 (2^6==64).
     * "The top _b-1_ bits are used for the input amount", 5
     * "the next-most-significant 2<sup>b-1</sup> bits are rotated right and used
       as the output", so we'll rotate right 2^5 bits, which is 32 bits.
     * "and the low 2<sup>b-1</sup>+1−b bits are discarded", so that's 32+1-6,
       which is our final answer of 27.

Al<i>right</i>, so we've _finally_ got our generator.

```rust
fn pcg_xsh_rr_1(state: &mut u64) -> u32 {
  // We'll use the LCG3 values I guess?
  let a = 69069;
  let c = 1;
  *state = state.wrapping_mul(a).wrapping_add(c);
  let mut x = *state;
  let rotation = (x >> 59) as u32;
  x ^= x >> 18;
  ((x >> 27) as u32).rotate_right(rotation)
}
```

Hey, they've got some example code. Oh, they've got a different multiplier (the
`a` value), and "an arbitrary odd constant" (the `c` value).

> The generator applies the output transformation to the _initial_ state rather
> than the _final_ state in order to increase the available instruction-level
> parallelism to maximize performance on modern superscalar processors.

Sounds fair.

```rust
fn pcg_xsh_rr_2(state: &mut u64) -> u32 {
  // These are Wikipedia's suggested values
  let a = 6364136223846793005;
  let c = 1442695040888963407; // this can be any odd const
  *state = state.wrapping_mul(a).wrapping_add(c);
  let mut x = state;
  let rotation = (x >> 59) as u32;
  x ^= x >> 18;
  ((x >> 27) as u32).rotate_right(rotation)
}
```

There's some comparisons with other PRNGs which seem convincing enough. Oh, and
a link to [the PCG website](http://www.pcg-random.org/). "What's Wrong with Your
Current RNG" oh no! "The PCG Family Is Better" oh my! Ah, there's [a whole
paper](http://www.pcg-random.org/paper.html) if you want to know more. I feel
like we've gone into it enough here.

As you can see, we just looked up a few things and we ended up with our own PRNG
implementation that we pretty much understand (well, if you read the full
Wikipedia articles you probably understand most of it).

Of course, we don't want to mix up our `u64` state value with other `u64`
values, so we'll make a struct for it and give it a method and all that jazz.

```rust
#[derive(Debug, Clone)]
struct PCG32 {
  state: u64,
}

impl Default for PCG32 {
  /// Makes a generator with the default state suggested by Wikipedia.
  fn default() -> Self {
    PCG32 { state: 0x4d595df4d0f33173 }
  }
}

impl PCG32 {
  fn next_u32(&mut self) -> u32 {
    const A: u64 = 6364136223846793005;
    const C: u64 = 1442695040888963407; // this can be any odd const
    self.state = self.state.wrapping_mul(A).wrapping_add(C);
    let mut x = self.state;
    let rotation = (x >> 59) as u32;
    x ^= x >> 18;
    ((x >> 27) as u32).rotate_right(rotation)
  }
}
```

So, for now, we'll automatically pick a seed from the system time. Normally, if
you care about your initialization being as random as possible, you _don't_ want
to do this, because the system clock is a _very_ low quality randomness source.
You'd want to ask your OS to please give you a random number from its CSPRNG.
We're not using `rand` so no one has abstracted way the OS differences there for
us (that's the best feature of `rand`). However, the default seed quality
doesn't matter, because later on we'll just outright let the player pick their
own random seed if they want. Like Brogue and Minecraft and such.

```rust
pub fn u64_from_time() -> u64 {
  use std::time::{SystemTime, UNIX_EPOCH};
  let the_duration = match SystemTime::now().duration_since(UNIX_EPOCH) {
    Ok(duration) => duration,
    Err(system_time_error) => system_time_error.duration(),
  };
  if the_duration.subsec_nanos() != 0 {
    the_duration.as_secs().wrapping_mul(the_duration.subsec_nanos() as u64)
  } else {
    the_duration.as_secs()
  }
}
```

And then throw one of these into our `GameState` and we're all set. The
`GameState` is getting kinda big, so we'll make a `GameState::new` method to
clean that up a bit. The only part that really changes is the seed, and then the
rest should play out from there, so we'll just pass a seed and then let the
`new` method solve the rest for us.

## Part 03b: We get back to doing a randomized map

So we're ready to generate random dungeons. What kind of dungeons should we
make? I put a mild notion of a plot into the README.md file since last week.

> You are **Kasidin**, a **Terulo**. You live a quiet life of eating **rocks** and
> drinking **spicy lava soup**. One day, the **Evil King Adlori** invaded the land
> of volcanos and shut off all of the lava. Now there is no more **spicy lava
> soup**! You must travel deep into the heart of the earth and defeat **Adlori**
> to restore the soup.

So, you're playing as Kasidin, and you go through the volcano lands to defeat
King Adlori. Well, to start off at least we'll want something like caves.

Ah, look, [there's a Roguebasin article for
that](http://www.roguebasin.com/index.php?title=Cellular_Automata_Method_for_Generating_Random_Cave-Like_Levels).

So, we'll be doing a lot of 2d manipulations, setting things on and off, so
we'll want to be able to access locations faster than doing a hash if possible,
and we'll know the exact size of the dungeon to create already, so we'll have
the cave generation create a `VecImage`, and then just copy it into the terrain
`HashMap`. We'll probably want to use the `VecImage` directly, but other code
assumes the `HashMap` deal, so we won't change that for the moment. Now the new
method looks like this:

```rust
  fn new(seed: u64) -> Self {
    let mut out = Self {
      player_location: Location { x: 5, y: 5 },
      creatures: HashMap::new(),
      terrain: HashMap::new(),
      gen: PCG32 { state: seed },
    };
    out.creatures.insert(Location { x: 5, y: 5 }, Creature {});
    //out.terrain.insert(Location { x: 10, y: 10 }, Terrain::Wall);
    let caves = make_cellular_caves(TILE_GRID_WIDTH, TILE_GRID_HEIGHT, &mut out.gen);
    for (x, y, tile) in caves.iter() {
      out
        .terrain
        .insert(Location { x: x as i32, y: y as i32 }, if *tile { Terrain::Wall } else { Terrain::Floor });
    }
    out
  }
```

and then of course we have this too:

```rust
fn make_cellular_caves(width: usize, height: usize, gen: &mut PCG32) -> VecImage<bool> {
  unimplemented!()
}
```

Now we just fill _that part_ in.
