# Week 02

FIRST THINGS FIRST. If you're doing the demo from last time, please use `cargo
update`. I've put out a `0.1.1` version of `dwarf_term`, which fixes two
different bugs that appeared when people started trying things out.

# Part 02: A Static Dungeon

So, the `'@'` moves anywhere we like. He is unconstrained, but by being able to
do anything, he also has nothing to do. Let's give him some limits.

First we will make a static dungeon shape, and then in the next part will we
make random dungeon shapes.

Let's start by moving stuff that isn't in the main function itself into
`lib.rs`, because rust projects normally like to split up the library and binary
portion of the code. This mostly involves marking a whole lot of stuff `pub`,
since rust has private types, fields, and functions by default.

So let's make a type to hold our game:

```rust
#[derive(Debug, Clone, Default)]
pub struct GameWorld {
  pub player_location: Location,
  pub creatures: HashMap<Location, Creature>,
  pub terrain: HashMap<Location, Terrain>,
}
```

A `GameWorld` has a player_location, some creatures, and some terrain. Seems
good enough for an `'@'` and at least one wall tile.

```rust
#[derive(Debug, Clone, Copy)]
pub struct Creature {}

#[derive(Debug, Clone, Copy)]
pub enum Terrain {
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
pub struct Location {
  pub x: i32,
  pub y: i32,
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
  pub fn move_player(&mut self, delta: Location) {
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
don't know the internal state, but it's still totally deterministic like normal
math. We're more technically using pesudo-random number generators (PRNG).
They've got some sort of state, and then they do some math, and then they change
their internal state and then decide on an output.

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
pub struct PCG32 {
  state: u64,
}

impl Default for PCG32 {
  /// Makes a generator with the default state suggested by Wikipedia.
  fn default() -> Self {
    PCG32 { state: 0x4d595df4d0f33173 }
  }
}

impl PCG32 {
  pub fn next_u32(&mut self) -> u32 {
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
  pub fn new(seed: u64) -> Self {
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

So now we take that function and write an outline:

```rust
fn make_cellular_caves(width: usize, height: usize, gen: &mut PCG32) -> VecImage<bool> {
  unimplemented!()
}
```

Okay so let's review the steps from roguebasin:

1. Start with every cell being 45% likely to be on.
2. Set the next stage at position `p` to be on if range 1 has 5 or more on, or
   if range 2 has 0 on.
3. Repeat step 2 five times.

So first we need a way to make a 45% percent chance. Our RNG can't do that, so
we'll need a new method or something. Let's look up how to [convert random u32
into random
float](https://www.google.com/search?q=convert+random+u32+into+random+float).
Most of this is garbage telling us how to call some library in whatever
language, but [there is a
paper](https://www.doornik.com/research/randomdouble.pdf) on the first page that
looks interesting. It also looks really technical. We don't need to do full
floats, we just need to roll a number from 1 to 100, and then see if that's less
than or equal to 45. So we'll hold off on full floats and do dice rolls. If we
use the modulus operator we can do `x % 100` to get a number in the range
`0..=99`, and if we add 1 we'll get `1..=100`. However, if we do that we'll
introduce non-uniformity, so we have to discard some results and roll again if
we get results that would make us non-uniform.

**Side note on the notation:** The `..=` operator is rust's
[RangeInclusive](https://doc.rust-lang.org/std/ops/struct.RangeInclusive.html)
operator, and there is also `..` for the
[Range](https://doc.rust-lang.org/std/ops/struct.Range.html) operator.
RangeInclusive is for when you want to do things like saying that a 6-sided die
is the range `1..=6`, or the byte values are `0..=255`, and exclusive range is
for when you want to do something like `for i in 0..arr.len() {`. Yes, it's
stupid that `Range` isn't called `RangeExclusive` to match `RangeInclusive`, but
the `Range` type was made as part of 1.0, when we were a lot stupider than we
are now, and the `RangeInclusive` type was made in 1.26, when we were almost as
smart as we are now. If you're wondering, "now" is 1.27.

So, since we have to get a range, and then decide what the upper bound is, and we
also have to do a lot of random rolls, we'll make a pre-computed random range
reusable.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RandRangeInclusive32 {
  base: u32,
  width: u32,
  reject: u32,
}
```

So we will work entirely in `u32` values because it's easier that way. We've got
the `base` value, which is what you get if `x % y` is 0, and the `width` value,
which is the `y` part of the `x % y`, and then the `reject` value, which is the
biggest value we can evenly fit into our target width. If you're past the reject
value we have to roll again. The math for this is kinda fiddly, with little +1s
and -1s, but it goes like this:

```rust
impl RandRangeInclusive32 {
  pub fn new(range_incl: RangeInclusive<u32>) -> Self {
    let (low, high) = range_incl.into_inner();
    assert!(low < high, "RandRangeInclusive32 must go from low to high, got {} ..= {}", low, high);
    let base = low;
    let width = (high - low) + 1;
    debug_assert!(width > 0);
    let width_count = ::std::u32::MAX / width;
    let reject = (width_count * width) - 1;
    RandRangeInclusive32 { base, width, reject }
  }

  /// Lowest possible result of this range.
  pub fn low(&self) -> u32 {
    self.base
  }

  /// Highest possible result of this range.
  pub fn high(&self) -> u32 {
    self.base + (self.width - 1)
  }

  /// Converts any `u32` into `Some(val)` if the input can be evenly placed
  /// into range, or `None` otherwise.
  pub fn convert(&self, roll: u32) -> Option<u32> {
    if roll > self.reject {
      None
    } else {
      Some(self.base + (roll % self.width))
    }
  }
}
```

Now, that seems error prone, so we'll write a test to go through every single `u32` value and check what we get.

```rust
#[test]
#[ignore]
pub fn range_range_inclusive_32_sample_validity_test_d6() {
  let the_range = RandRangeInclusive32::new(1..=6);
  let mut outputs: [u32; 7] = [0; 7];
  // 0 to one less than max
  for u in 0..::std::u32::MAX {
    let opt = the_range.convert(u);
    match opt {
      Some(roll) => outputs[roll as usize] += 1,
      None => outputs[0] += 1,
    };
  }
  // max
  let opt = the_range.convert(::std::u32::MAX);
  match opt {
    Some(roll) => outputs[roll as usize] += 1,
    None => outputs[0] += 1,
  };
  assert!(outputs[0] < 6);
  let ones = outputs[1];
  assert_eq!(ones, outputs[2], "{:?}", outputs);
  assert_eq!(ones, outputs[3], "{:?}", outputs);
  assert_eq!(ones, outputs[4], "{:?}", outputs);
  assert_eq!(ones, outputs[5], "{:?}", outputs);
  assert_eq!(ones, outputs[6], "{:?}", outputs);
}

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

Okay, it's gonna be a really slow test, so we'll mark it as ignore, and then
only run it when we're going over all our tests, and be sure to run it in
release mode.

```
D:\dev\roguelike-tutorial-2018>cargo test --release -- --ignored
    Finished release [optimized] target(s) in 6.96s
     Running target\release\deps\roguelike_tutorial_2018-af51f147971e8dbf.exe

running 1 test
test range_range_inclusive_32_sample_validity_test_d6 ... ok

test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out

     Running target\release\deps\kasidin-258d2a1c4b30463a.exe
   Doc-tests roguelike-tutorial-2018

running 0 tests

test result: ok. 0 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out
```

Alright, we're in business. Also, we'll want a way to use a generator to reroll
until we get an output.

```rust
  pub fn roll_with(&self, gen: &mut PCG32) -> u32 {
    loop {
      if let Some(output) = self.convert(gen.next_u32()) {
        return output;
      }
    }
  }
```

So now we can fill our initial cells

```rust
fn make_cellular_caves(width: usize, height: usize, gen: &mut PCG32) -> VecImage<bool> {
  let d100 = RandRangeInclusive32::new(1..=100);
  let mut buffer_a: VecImage<bool> = VecImage::new(width, height);
  let mut buffer_b: VecImage<bool> = VecImage::new(width, height);
  // fill the initial buffer, all cells 45% likely.
  for (_x,_y,mut_ref) in buffer_a.iter_mut(){
    if d100.roll_with(gen) <= 45 {
      *mut_ref = true;
    }
  }
  unimplemented!()
}
```

Now we need a way to count the tiles at a given range. We can define this inside
the `make_cellular_caves` function, since it'll only be used inside that
function.

```rust
  let range_count = |buf: &VecImage<bool>, x: usize, y: usize, range: u32| {
    debug_assert!(range > 0);
    let mut total = 0;
    for y in ((y as isize - range as isize) as usize)..=(y + range as usize) {
      for x in ((x as isize - range as isize) as usize)..=(x + range as usize) {
        if y == 0 && x == 0 {
          continue;
        } else {
          match buf.get((x, y)) {
            Some(&b) => if b {
              total += 1;
            },
            None => {
              total += 1;
            }
          }
        }
      }
    }
    total
  };
```

So there's two tricky things here. We might try to check the range of an out of
bounds location, so we have to use the `get` method for bounds-safe checking.
Then, even with that done, what do we do if we did go out of bounds? Well, for
now we'll just count it as being a wall, and hope it gives good results. We'll
see when we've got something to see.

So, armed with this, we need to fill buffer b based on buffer a, and then fill
buffer a based on buffer b, back and forth. Sounds like another inner function.

```rust
  let cave_copy = |dest: &mut VecImage<bool>, src: &VecImage<bool>| {
    for (x, y, mut_ref) in dest.iter_mut() {
      // TODO: this will count up some of the cells more than once, perhaps we
      // can make this more efficient by making it more fiddly.
      *mut_ref = range_count(src, x, y, 1) >= 5 || range_count(src, x, y, 2) <= 1;
    }
  };
```

and we use that

```rust
  // cave copy from A into B, then the reverse, 5 times total
  cave_copy(&buffer_a, &mut buffer_b);
  cave_copy(&buffer_b, &mut buffer_a);
  cave_copy(&buffer_a, &mut buffer_b);
  cave_copy(&buffer_b, &mut buffer_a);
  cave_copy(&buffer_a, &mut buffer_b);
  // the final work is now in B
  buffer_b
}
```

![it-works-marty](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week02-01.png)

Hmm, we're getting those pockets that they talked about. Ah, if we look closer
there's an alternate ruleset that helps make smoother caves. I like the jagged
parts and random pillars, we just want to ensure that we're connected. Looks
like if we do a flood fill based copy at the end we can ensure that we're
connected. If we don't get enough connected tiles, we'll just do it all again.

```rust
  'work: loop {
    // fill the initial buffer, all cells 45% likely.
    for (_x, _y, mut_ref) in buffer_a.iter_mut() {
      if d100.roll_with(gen) <= 45 {
        *mut_ref = true;
      }
    }
    // cave copy from A into B, then the reverse, 5 times total
    cave_copy(&buffer_a, &mut buffer_b);
    cave_copy(&buffer_b, &mut buffer_a);
    cave_copy(&buffer_a, &mut buffer_b);
    cave_copy(&buffer_b, &mut buffer_a);
    cave_copy(&buffer_a, &mut buffer_b);
    // good stuff is in B, flood copy back into A
    let copied_count = flood_copy(&buffer_b, &mut buffer_a, gen);
    if copied_count >= (width * height) / 2 {
      return buffer_a;
    } else {
      continue 'work;
    }
  }
```

Unfortunately, the flood copy itself is quite long.

```rust
  let flood_copy = |src: &VecImage<bool>, dest: &mut VecImage<bool>, gen: &mut PCG32| {
    dest.set_all(true);
    let mut copied_count = 0;
    let start = {
      let d_width = RandRangeInclusive32::new(0..=(width as u32));
      let d_height = RandRangeInclusive32::new(0..=(height as u32));
      let mut x = d_width.roll_with(gen) as usize;
      let mut y = d_height.roll_with(gen) as usize;
      let mut tries = 0;
      while src[(x, y)] {
        x = d_width.roll_with(gen) as usize;
        y = d_height.roll_with(gen) as usize;
        tries += 1;
        if tries > 100 {
          return 0;
        }
      }
      (x, y)
    };
    let mut open_set = HashSet::new();
    let mut closed_set = HashSet::new();
    open_set.insert(start);
    while !open_set.is_empty() {
      let loc: (usize, usize) = *open_set.iter().next().unwrap();
      open_set.remove(&loc);
      if closed_set.contains(&loc) {
        continue;
      } else {
        closed_set.insert(loc);
      };
      dest[loc] = false;
      copied_count += 1;
      if loc.0 > 1 {
        open_set.insert((loc.0 - 1, loc.1));
      }
      if loc.0 < (src.width() - 1) {
        open_set.insert((loc.0 + 1, loc.1));
      }
      if loc.1 > 1 {
        open_set.insert((loc.0, loc.1 - 1));
      }
      if loc.1 < (src.height() - 1) {
        open_set.insert((loc.0, loc.1 + 1));
      }
    }
    copied_count
  };
```

And turn it on...

![whoops](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week02-02.png)

Okay, not supposed to be like that for sure. There's two things wrong.

1. We tried to not copy over at the edge so that the cave result will never run
   up against the actual map bounds. That sure didn't work for the upper bounds
2. We didn't copy the data.

```rust
      if loc.0 > 1 {
        open_set.insert((loc.0 - 1, loc.1));
      }
      if loc.0 < (src.width() - 2) {
        open_set.insert((loc.0 + 1, loc.1));
      }
      if loc.1 > 1 {
        open_set.insert((loc.0, loc.1 - 1));
      }
      if loc.1 < (src.height() - 2) {
        open_set.insert((loc.0, loc.1 + 1));
      }
```

![better](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week02-03.png)

Okay, good bounds there, but we're not reading the source.

```rust
      dest[loc] = src[loc];
```

![better](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week02-04.png)

Nope, that's not right, that spreads too much and we just get a full copy.

```rust
      if !src[loc] {
        dest[loc] = false;
        copied_count += 1;
        if loc.0 > 1 {
          open_set.insert((loc.0 - 1, loc.1));
        }
        if loc.0 < (src.width() - 2) {
          open_set.insert((loc.0 + 1, loc.1));
        }
        if loc.1 > 1 {
          open_set.insert((loc.0, loc.1 - 1));
        }
        if loc.1 < (src.height() - 2) {
          open_set.insert((loc.0, loc.1 + 1));
        }
      }
```

Whoops, something did a panic. Hmm. What could be out of bounds? Oh, right, the
initial location picking can be out of bounds.

```rust
      let d_width = RandRangeInclusive32::new(0..=((width - 1) as u32));
      let d_height = RandRangeInclusive32::new(0..=((height - 1) as u32));
```

Except sometimes now it hangs forever. Huh. Well, we throw in some print
statements in to diagnose what's going on... turns out that if it copies too few
tiles (because it copied a small pocket for example) and then resets... we just
loop forever. Oh, right, because buffer_a was assumed to be blank. Okay so we'll
just force it false or true.

```rust
    // fill the initial buffer, all cells 45% likely.
    for (_x, _y, mut_ref) in buffer_a.iter_mut() {
      *mut_ref = d100.roll_with(gen) <= 45;
    }
```

And now we finally get results with no spare pockets.

![final](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week02-05.png)

You can even see places where there might have been pockets that didn't get
copied into the final thing thanks to the flood_copy.

![final-remix](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week02-06.png)

## Part 03c: Scrolling Camera

Now the final step is to go just a little beyond and make a camera that scrolls
so that we can have dungeons of any size we want. First we make the dungeons 100
by 100 (just to pick a size).

![100x100](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week02-06.png)

Then we offset the screen position by the player's position.

```rust
        let loc_for_this_screen_position = Location {
          x: scr_x as i32 + game.player_location.x,
          y: scr_y as i32 + game.player_location.y,
        };
```

![100x100-whoops](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week02-06.png)

Okay, my bad, what I of course meant to say was that we offset it by the
player's location _minus_ half the display region (so that the player is always
kept in the middle of the screen).

```rust
      let offset = game.player_location - Location {
        x: (fgs.width() / 2) as i32,
        y: (fgs.height() / 2) as i32,
      };
      for (scr_x, scr_y, id_mut) in ids.iter_mut() {
        let loc_for_this_screen_position = Location {
          x: scr_x as i32,
          y: scr_y as i32,
        } + offset;
```

![100x100-success](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week02-06.png)

And let's make sure that the player always is placed on a floor tile to start. Let's add a method to `GameState`

```rust
  pub fn pick_random_floor(&mut self) -> Location {
    let indexer = RandRangeInclusive32::new(0..=99);
    let mut tries = 0;
    let mut x = indexer.roll_with(&mut self.gen) as usize;
    let mut y = indexer.roll_with(&mut self.gen) as usize;
    let mut loc = Location { x: x as i32, y: y as i32 };
    while self.terrain[&loc] != Terrain::Floor {
      x = indexer.roll_with(&mut self.gen) as usize;
      y = indexer.roll_with(&mut self.gen) as usize;
      loc = Location { x: x as i32, y: y as i32 };
      if tries > 5000 {
        panic!("couldn't find a floor tile!");
      }
    }
    loc
  }
```

and then update how `new` works

```rust
  pub fn new(seed: u64) -> Self {
    let mut out = Self {
      player_location: Location { x: 5, y: 5 },
      creatures: HashMap::new(),
      terrain: HashMap::new(),
      gen: PCG32 { state: seed },
    };
    let caves = make_cellular_caves(100, 100, &mut out.gen);
    for (x, y, tile) in caves.iter() {
      out
        .terrain
        .insert(Location { x: x as i32, y: y as i32 }, if *tile { Terrain::Wall } else { Terrain::Floor });
    }

    let player_start = out.pick_random_floor();
    out.creatures.insert(player_start, Creature {});
    out.player_location = player_start;

    out
  }
```

And there we go, random starting locations with a scrolling camera.
