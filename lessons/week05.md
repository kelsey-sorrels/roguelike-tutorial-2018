# Week 05

## Part 08: Items and Inventory

So this part is about "items and inventory", but to start we'll just do two
kinds of potions (health potions and damage potions). In part 09 we'll add
another type of non-equipment item, and then in later weeks we'll add equipment
eventually.

### Place 08.a: Placing Items And Displaying Them

So the first thing that we need to do is declare a type for items. An item is..
probably one of several situations, like you find a potion or a bomb or a sword
or whatever. Depending on what the item's particulars are, then it'll have
different fields we care about. We could use a single struct for every item and
then have some really general fields, but we'll try an `enum` this time around
because I don't think that we've used one yet.

```rust
#[derive(Debug, PartialEq, Eq, Hash)]
pub enum Item {
  PotionHealth,
  PotionDamage,
}
```

An Item is either a Potion of Health or a Potion of Strength. Seems fine. We can
add more types of item later, and if there's enough types we can reorganize it
all later. Enums and enum matches are all checked by the compiler, so if we make
changes to the type there's no fear of compiling code that'll mysteriously
crash. There's all the _normal_ fear of having to go change code in a lot of
places, but you can't ever escape that one when you're changing your data layout.

Next we're gonna add a way for the dungeon to store items and for a creature to
store items. Thankfully, an empty Vec is basically free (it's the size of 3
`usize` values, but it doesn't do a heap allocation until you actually add the
first element). In other words, we'll just use Vec for Items.

```rust
#[derive(Debug, Default)]
pub struct GameWorld {
  pub player_location: Location,
  pub creature_list: Vec<Creature>,
  pub creature_locations: HashMap<Location, CreatureID>,
  pub item_locations: HashMap<Location, Vec<Item>>,
  pub terrain: HashMap<Location, Terrain>,
  pub gen: PCG32,
}
```

This is pretty obvious, we're just adding one new HashMap into the mix.

```rust
#[derive(Debug)]
pub struct Creature {
  pub icon: u8,
  pub color: u32,
  pub is_the_player: bool,
  pub id: CreatureID,
  pub hit_points: i32,
  pub damage_step: i32,
  pub inventory: Vec<Item>,
}
```

And the same thing here, we just throw on an extra field. In both cases we have
to adjust the `new` method as well to add a default value when the types are
created.

Finally, we put a bunch of potions all over the map. By now you can probably
write this part before I even tell you what I did, since we've been working with
HashMaps enough.

```rust
    // add some items
    for _ in 0..50 {
      let item_spot = out.pick_random_floor();
      let new_item = if (out.gen.next_u32() as i32) < 0 {
        Item::PotionHealth
      } else {
        Item::PotionStrength
      };
      out.item_locations.entry(item_spot).or_insert(Vec::new()).push(new_item);
    }
```

Finally, we have to display all the items to the camera. Our strategy here will
match the "normal" roguelike. Creatures get highest priority to be drawn, then
items, then terrain.

```rust
      for (scr_x, scr_y, id_mut) in ids.slice_mut((0, 0)..map_view_end).iter_mut() {
        let loc_for_this_screen_position = Location {
          x: scr_x as i32,
          y: scr_y as i32,
        } + offset;
        if seen_set.contains(&(loc_for_this_screen_position.x, loc_for_this_screen_position.y)) {
          match game.creature_locations.get(&loc_for_this_screen_position) {
            Some(cid_ref) => {
              let creature_here = game
                .creature_list
                .iter()
                .find(|&creature_ref| &creature_ref.id == cid_ref)
                .expect("Our locations and list are out of sync!");
              *id_mut = creature_here.icon;
              fgs[(scr_x, scr_y)] = creature_here.color;
            }
            None => match game.item_locations.get(&loc_for_this_screen_position) {
              Some(item_vec_ref) => match item_vec_ref.get(0) {
                Some(Item::PotionHealth) => {
                  *id_mut = POTION_GLYPH;
                  fgs[(scr_x, scr_y)] = rgb32!(250, 5, 5);
                }
                Some(Item::PotionStrength) => {
                  *id_mut = POTION_GLYPH;
                  fgs[(scr_x, scr_y)] = rgb32!(5, 240, 20);
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
              },
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
            },
          }
        } else {
          *id_mut = b' ';
        }
      }
```

OH BOY. That is not so nice code. Soon enough we'll probably want to add color
and glyph data to things in the game instead of having the camera do it all. Or
even if we do store it all in the camera, we'll still label the colors more
perhaps.

And we've got that annoying thing where we have to have a block for drawing the
terrain twice because we might end up not having an item to draw twice. Hmm, I
bet we can at least cut out that much. We just have to merge the chance of an
item vec being there and then looking up the 0th index into a single expression.

```rust
          match game.creature_locations.get(&loc_for_this_screen_position) {
            Some(cid_ref) => {
              let creature_here = game
                .creature_list
                .iter()
                .find(|&creature_ref| &creature_ref.id == cid_ref)
                .expect("Our locations and list are out of sync!");
              *id_mut = creature_here.icon;
              fgs[(scr_x, scr_y)] = creature_here.color;
            }
            None => match game
              .item_locations
              .get(&loc_for_this_screen_position)
              .and_then(|item_vec_ref| item_vec_ref.get(0))
            {
```

And, I think that we can make the whole thing a lot _visually_ cleaner if we
just make the match pick the glyph and color and then do the assignment outside
of the match. It's the same code really, but you can tell a little better that
all we're doing regardless of branch is picking out what to draw and then
drawing that into our buffer.

```rust
        let (glyph, color) = if seen_set.contains(&(loc_for_this_screen_position.x, loc_for_this_screen_position.y)) {
          match game.creature_locations.get(&loc_for_this_screen_position) {
            Some(cid_ref) => {
              let creature_here = game
                .creature_list
                .iter()
                .find(|&creature_ref| &creature_ref.id == cid_ref)
                .expect("Our locations and list are out of sync!");
              (creature_here.icon, creature_here.color)
            }
            None => match game
              .item_locations
              .get(&loc_for_this_screen_position)
              .and_then(|item_vec_ref| item_vec_ref.get(0))
            {
              Some(Item::PotionHealth) => (POTION_GLYPH, rgb32!(250, 5, 5)),
              Some(Item::PotionStrength) => (POTION_GLYPH, rgb32!(5, 240, 20)),
              None => match game.terrain.get(&loc_for_this_screen_position) {
                Some(Terrain::Wall) => (WALL_TILE, rgb32!(155, 75, 0)),
                Some(Terrain::Floor) => (b'.', rgb32!(128, 128, 128)),
                None => (b' ', 0),
              },
            },
          }
        } else {
          (b' ', 0)
        };
        *id_mut = glyph;
        fgs[(scr_x, scr_y)] = color;
```

Yeah, I like that a lot better.

So now we can turn on the game and see some green and red potions on the ground.

![potions1](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week05-01.png)

### Place 08.b: Auto-pickup

Now we just need to change it so that if the player steps into a place with an
item they also grab the item as part of that. There's no inventory limit, and it
doesn't take a turn, so I don't think we need a way to disable the auto-pickup.

We just go to `GameWorld::move_player`,

```rust
          Terrain::Floor => {
            let player_id = self
              .creature_locations
              .remove(&self.player_location)
              .expect("The player wasn't where they should be!");
            let old_creature = self.creature_locations.insert(player_move_target, player_id);
            debug_assert!(old_creature.is_none());
            self.player_location = player_move_target;
          }
```

And after we've updated the player's location, we add a little more.

```rust
            // grab items that are here, if any
            let player_id_ref = self.creature_locations.get(&self.player_location).unwrap();
            let player_mut = self
              .creature_list
              .iter_mut()
              .find(|creature_mut| &creature_mut.id == player_id_ref)
              .unwrap();
            let floor_items = self.item_locations.entry(self.player_location).or_insert(Vec::new());
            player_mut.inventory.append(floor_items);
```

That's all there is to it.

### Place 08.c: Cataloging Items

TODO: A way to display the inventory.

### Place 08.d: Drinking Items

TODO: drink a potion when you press its letter while the inventory is open.

### Place 08.e: Equipping Items

Ha, you thought that we'd do all the basic item stuff at the same time? How foolish!

Of course we have to wait until Part 13 to actually _equip_ an item.

Doing the tutorial out of order would simply be heretical.

## Part 09: Ranged Attacks and Targeting

TODO: Preview

### Part 09.a: New Item Type (bombs)

TODO: Coding

### Part 09.b: Equipping bombs

TODO: Coding

### Part 09.c: Throwing a bomb

TODO: Coding
