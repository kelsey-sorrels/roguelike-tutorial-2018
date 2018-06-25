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



# Part 03: A Random Dungeon

TODO: make a random map
