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

![potions](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week05-01.png)

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

So now we want to display the current inventory. To do this we want to make a
concept of the user's display being in different modes.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DisplayMode {
  Game,
  Inventory
}
```

Now we track that in the main portion of the binary.

```rust
  // Main loop
  let mut running = true;
  let mut pending_keys = vec![];
  let mut display_mode = DisplayMode::Game; // this line is new
  'game: loop {
```

And then when we process keys, we now first split on the current display mode
we're in.

```rust
    for key in pending_keys.drain(..) {
      match display_mode {
        DisplayMode::Game => match key {
          VirtualKeyCode::Up => game.move_player(Location { x: 0, y: 1 }),
          VirtualKeyCode::Down => game.move_player(Location { x: 0, y: -1 }),
          VirtualKeyCode::Left => game.move_player(Location { x: -1, y: 0 }),
          VirtualKeyCode::Right => game.move_player(Location { x: 1, y: 0 }),
          VirtualKeyCode::I => display_mode = DisplayMode::Inventory,
          _ => {}
        },
        DisplayMode::Inventory => match key {
          VirtualKeyCode::Escape => display_mode = DisplayMode::Game,
          _ => {}
        },
      }
    }
```

Ah, but now the Escape key is used for both closing the window as a whole, and
also the inventory screen. Let's stop Escape from closing the window. We still
have to follow `WindowEvent::CloseRequested` to make sure that our program
responds properly to people clicking the `X` in the upper left and things like
that (depending on the GUI system our window is in). We can even eliminate the
TODO in the `if` block just below the `poll_events` call.

```rust
    // Grab all new presses
    term.poll_events(|event| match event {
      Event::WindowEvent { event: win_event, .. } => match win_event {
        WindowEvent::CloseRequested => {
          running = false;
        }
        WindowEvent::KeyboardInput {
          input: KeyboardInput {
            state: ElementState::Pressed,
            virtual_keycode: Some(key),
            ..
          },
          ..
        } => {
          pending_keys.push(key);
        }
        _ => {}
      },
      _ => {}
    });
```

And then we need to also branch the draw code based on what display is active.
Each path of this will be big enough that we should probably make it some
function calls.

```rust
    const FOV_DISPLAY_RANGE: i32 = TILE_GRID_WIDTH as i32 / 2; // assumes that the display is wider than tall
    let mut seen_set = HashSet::new();
    ppfov(
      (game.player_location.x, game.player_location.y),
      FOV_DISPLAY_RANGE,
      |x, y| game.terrain.get(&Location { x, y }).map(|&t| t == Terrain::Wall).unwrap_or(true),
      |x, y| drop(seen_set.insert((x, y))),
    );
    {
      match display_mode {
        DisplayMode::Game => draw_game(&mut term, &game, &seen_set),
        DisplayMode::Inventory => draw_inventory(&mut term, &game),
      }
    }
```

They've got signatures like this

```rust
fn draw_game(term: &mut DwarfTerm, game: &GameWorld, seen_set: &HashSet<(i32, i32)>) {}

fn draw_inventory(term: &mut DwarfTerm, game: &GameWorld) {}
```

And `draw_game` is just everything we had before. Scans FOV, displays things on
the screen that are in FOV, all that.

`draw_inventory` is a new thing. We have to decide how we wanna draw anything. I
think that we'll stack up items as much as possible, and then display the
results. This part has a few steps but each step is mostly simple.

First we figure out how many of each item the player has:

```rust
fn draw_inventory(term: &mut DwarfTerm, game: &GameWorld) {
  let (mut _fgs, mut _bgs, mut ids) = term.layer_slices_mut();
  ids.set_all(0);

  let mut map_item_count = HashMap::new();
  for item_ref in game
    .creature_list
    .iter()
    .find(|creature_ref| creature_ref.is_the_player)
    .unwrap()
    .inventory
    .iter()
  {
    *map_item_count.entry(item_ref).or_insert(0) += 1;
  }
```

Then we make a list of strings to display from that.

```rust
  let mut item_list = vec![];
  for (key, val) in map_item_count.into_iter() {
    match val {
      0 => panic!("what the heck?"),
      1 => item_list.push(format!("{:?}", key)),
      count => item_list.push(format!("{:?} ({})", key, count)),
    }
  }
```

Then we write out each item one at a time. This part uses some `unsafe` code, so
again we have to be careful about it.

```rust
  let mut the_y_position: isize = ids.height() as isize - 1;
  for (i, item) in item_list.into_iter().enumerate() {
    if the_y_position < 0 {
      break;
    }
    let mut this_line_slice_mut: &mut [u8] =
      unsafe { ::std::slice::from_raw_parts_mut(ids.as_mut_ptr().offset(ids.pitch() * the_y_position), ids.width()) };

    let letter = i + ('a' as u8 as usize);

    write!(this_line_slice_mut, "{}) {}", letter, item).ok();

    the_y_position -= 1;
  }
```

Okay, so we turn it on, press `'i'`, and then the screen goes blank... Right,
because if the inventory is empty, we don't have anything to indicate that to
the player. Of course. So we'll have a line that says you've got the inventory
open, and if you've got no items it'll say that too.

```rust
  // draw the menu title
  {
    let menu_title = "== Inventory ==";
    assert!(menu_title.len() < ids.width());
    let x_offset = (ids.width() - menu_title.len()) as isize / 2;
    let y_offset = (ids.height() as isize - 1) as isize;
    let mut this_line_slice_mut: &mut [u8] =
      unsafe { ::std::slice::from_raw_parts_mut(ids.as_mut_ptr().offset(x_offset + y_offset * ids.pitch()), menu_title.len()) };
    write!(this_line_slice_mut, "{}", menu_title).ok();
  }
  // draw the items
  if item_list.len() > 0 {
    let mut the_y_position: isize = ids.height() as isize - 2;
    for (i, item) in item_list.into_iter().enumerate() {
      if the_y_position < 0 {
        break;
      }
      let mut this_line_slice_mut: &mut [u8] =
        unsafe { ::std::slice::from_raw_parts_mut(ids.as_mut_ptr().offset(ids.pitch() * the_y_position), ids.width()) };
      let letter = i + ('a' as u8 as usize);
      write!(this_line_slice_mut, "{}) {}", letter, item).ok();
      the_y_position -= 1;
    }
  } else {
    let message = "You have no items on hand.";
    assert!(message.len() < ids.width());
    let x_offset = (ids.width() - message.len()) as isize / 2;
    let y_offset = (ids.height() as isize - 3) as isize;
    let mut this_line_slice_mut: &mut [u8] =
      unsafe { ::std::slice::from_raw_parts_mut(ids.as_mut_ptr().offset(x_offset + y_offset * ids.pitch()), message.len()) };
    write!(this_line_slice_mut, "{}", message).ok();
  }
```

![inventory-wrong](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week05-02.png)

Bit of a goof there, we don't clear the foreground colors. In fact we should
generally clear the foreground and background at the start of draw cycles if
we're not gonna use the full space. For here and for the status line in the game
draw, and things like that.

Now it shows an empty inventory properly. Let's grab a potion and see what it does.

![inventory-wrong2](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week05-03.png)

Ah, the next goof is that we don't cast `letter` into a `char` when we format
it. Also, we're using a `Debug` string to show it, which is just the base name
of the Item variant. We can add a `Display` impl to `Item`, which is the
intended trait for "user facing" text forms of a data type.

```rust
// goes back in the lib.rs

impl ::std::fmt::Display for Item {
  fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
    match self {
      Item::PotionHealth => write!(f, "Potion of Restore Health"),
      Item::PotionStrength => write!(f, "Potion of Gain Strength"),
    }
  }
}
```

And we change our item listing to use the "display" formatter instead of "debug"
by changing `{:?}` to be `{}`

```rust
  let mut item_list = vec![];
  for (key, val) in map_item_count.into_iter() {
    match val {
      0 => panic!("what the heck?"),
      1 => item_list.push(format!("{}", key)),
      count => item_list.push(format!("{} ({})", key, count)),
    }
  }
```

![inventory-is-better](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week05-04.png)

However, there's now another problem. If you pick up both kinds of items and
open the inventory it'll go crazy. What's happening? Every frame it's making a
new HashMap, and so when we iterate through that built up map things come out in
a random order. Each map keeps its own private RNG, so even two maps built up in
the same way can end up different. Terrible! Instead of using just a HashMap and
then throwing that into a Vec, we have to sort it somehow. There's two main
options:

* Make the HashMap into a Vec, then sort that Vec, then process it in a second
  pass now that it's sorted.
* Switch to the
  [BTreeMap](https://doc.rust-lang.org/std/collections/struct.BTreeMap.html)
  type, which is a similar mapping type, except it uses `Ord` instead of `Hash`,
  so the keys are kept in sorted order to begin with.

As usual, we'll try something new just to try it.

```rust
  let mut map_item_count = BTreeMap::new();
```

And we need to have `Ord` on the `Item` type, but that's no trouble since we can
just add it to the list of things to derive.

```rust
#[derive(Debug, PartialEq, Eq, Hash, PartialOrd, Ord)]
pub enum Item {
  PotionHealth,
  PotionStrength,
}
```

Aaaannnnddd..... that's it. That's _all_ we had to change. Neat.

### Place 08.d: Drinking Items

So now we'll make the potion of health and potion of gain strength actually have
an effect.

First, let's review a moment. All creatures start with 10 hit points and a
damage step of 5. This goes for the player and also the little `k` creatures. I
don't think I've named them so far. We'll say they're _kestrels_ I guess,
because kobolds are a little over used.

I think the player should be a little tougher than a kestrel even at the start.
We'll say that the player gets a little more hp and damage, while the kestrel
gets a little less. Let's codify this by adding `new_player` and `new_kestrel`
methods on the `Creature` type, which will each make a creature with the right
values. The core `new` method will also be there, if you need to make some
custom creature, but the defaults for hp and damage will go down to 1.

```rust
impl Creature {
  fn new(icon: u8, color: u32) -> Self {
    Creature {
      icon,
      color,
      is_the_player: false,
      id: CreatureID::atomic_new(),
      hit_points: 1,
      damage_step: 1,
      inventory: vec![],
    }
  }

  fn new_player() -> Self {
    let mut out = Self::new(b'@', TERULO_BROWN);
    out.is_the_player = true;
    out.hit_points = 20;
    out.damage_step = 5;
    out
  }

  fn new_kestrel() -> Self {
    let mut out = Self::new(b'k', KESTREL_RED);
    out.hit_points = 8;
    out.damage_step = 3;
    out
  }
}
```

And we call them at the appropriate places and all that and such.

Now as for what the effects of the potions will be.

* The potion of Gain Strength can just add 1 do your damage step. Easy.
* The potion of Restore Health will add back some hit points, but probably a
  random amount. We'll go with step 8 (the first step with two dice, because two
  dice gives more bell curved results). It also maybe shouldn't give out more
  hit points than some upper limit. We'll just cap it at 30 for now, that's
  probably fine.

While we're at it, we'll rename `step4` to be just `step`. That was a bit of a
goof on my part, we don't care about the "4" part for this game.

```rust
fn apply_potion(potion: &Item, target: &mut Creature, rng: &mut PCG32) {
  match potion {
    Item::PotionHealth => target.hit_points = (target.hit_points + step(rng, 8)).min(30),
    Item::PotionStrength => target.damage_step += 1,
  }
}
```

Now we just make the letters shown to items in the inventory. This one is rather
annoying to write because we have to write a stupid huge utility predicate.

```rust
fn letter_of(keycode: VirtualKeyCode) -> Option<char> {
  match keycode {
    VirtualKeyCode::A => Some('a'),
    VirtualKeyCode::B => Some('b'),
    VirtualKeyCode::C => Some('c'),
    VirtualKeyCode::D => Some('d'),
    VirtualKeyCode::E => Some('e'),
    VirtualKeyCode::F => Some('f'),
    VirtualKeyCode::G => Some('g'),
    VirtualKeyCode::H => Some('h'),
    VirtualKeyCode::I => Some('i'),
    VirtualKeyCode::J => Some('j'),
    VirtualKeyCode::K => Some('k'),
    VirtualKeyCode::L => Some('l'),
    VirtualKeyCode::M => Some('m'),
    VirtualKeyCode::N => Some('n'),
    VirtualKeyCode::O => Some('o'),
    VirtualKeyCode::P => Some('p'),
    VirtualKeyCode::Q => Some('q'),
    VirtualKeyCode::R => Some('r'),
    VirtualKeyCode::S => Some('s'),
    VirtualKeyCode::T => Some('t'),
    VirtualKeyCode::U => Some('u'),
    VirtualKeyCode::V => Some('v'),
    VirtualKeyCode::W => Some('w'),
    VirtualKeyCode::X => Some('x'),
    VirtualKeyCode::Y => Some('y'),
    VirtualKeyCode::Z => Some('z'),
    _ => None,
  }
}
```

Then we can adjust the key processing for when the inventory is open

```rust
        DisplayMode::Inventory => match key {
          VirtualKeyCode::Escape => display_mode = DisplayMode::Game,
          other => {
            letter_of(other).map(|ch| {
              if ch.is_alphabetic() && game.use_item(ch) {
                display_mode = DisplayMode::Game;
              }
            });
          }
        },
```

So now we have this `use_item` method we're expecting, and it returns `true` if
the item requested actually got used (thus taking a turn). We'll have to adjust
this control flow later if we want the ability to see descriptions or something
like that, but for now it's fine. If we wanted item description display we'd
just do that as its own screen, and then one of the options on that screen might
be a use button, or something similar.

```rust
// part of `impl GameWorld` in `lib.rs`
  pub fn use_item(&mut self, item_letter: char) -> bool {
    false
  }
```

Now we need to map the player's letter input into what item is being used. Hmm.
This won't be pretty because our data is poorly organized.

So, right now we've got items in inventory and then the display is stacking up
the items when it tells the player what they have. However, the inventory isn't
_also_ keeping items in stacks, so we have to do this double work. if there was
the time to do it, we'd want to make the Inventory hold a series of
InventoryStack elements and then when you add an Item into an Inventory it finds
the first stack that the item fits into, and pushes a new stack if it doesn't
fit any current stacks. However we're busy folk, and I honestly just don't want
to go back right now, so we'll struggle on for a bit like this.

Hmm, and the natural thing one might expect would be to get a reference to an
item, then use `apply_potion` with that item reference on the player. However,
the item reference would have a lifetime linked to the Creature of the player,
and the mutable player reference would also be off that same creature. So
there's really no way to do that. Two ways we could get around this: quick and
dirty path is to make `Item` be `Clone`. The harder path is to do some juggling
to remove the item from the player's inventory and then move it into
`apply_potion`. I'm really on the fence about this, because it seems like maybe
we should attack the second route, but we've piled on enough trouble by not
really having a good inventory API, so we'll make `Item` by `Copy` for now.

```rust
  pub fn use_item(&mut self, item_letter: char) -> bool {
    let player_mut = self.creature_list.iter_mut().find(|creature_ref| creature_ref.is_the_player).unwrap();
    let item_to_use = {
      let mut cataloged_inventory = BTreeMap::new();
      for item_ref in player_mut.inventory.iter() {
        *cataloged_inventory.entry(item_ref).or_insert(0) += 1;
      }
      let letter_index = item_letter as u8 - 'a' as u8;
      cataloged_inventory.into_iter().nth(letter_index as usize).map(|(&item, _count)| item)
    };
    match item_to_use {
      Some(item) => {
        apply_potion(&item, player_mut, &mut self.gen);
        for i in 0..player_mut.inventory.len() {
          if player_mut.inventory[i] == item {
            player_mut.inventory.remove(i);
            break;
          }
        }
        true
      }
      None => false,
    }
  }
```

### Place 08.e: Equipping Items

Ha, you thought that we'd do all the basic item stuff at the same time? How foolish!

Of course we have to wait until Part 13 to actually _equip_ an item.

Doing the tutorial out of order would simply be heretical.

In other words, we're done with part 8.

[current code](https://github.com/Lokathor/roguelike-tutorial-2018/tree/ec225764304f3a9c7809118d69126df217b8ad8a)

## Part 09: Ranged Attacks and Targeting

So now Kasidin needs a ranged attack. Except, they're not some sort of rogueish
human that would use a Scroll of Lighting, they're a tough rock brother! So
instead we'll be adding the classic weapon of the rock brothers, a Bomb! Let's
even go for two kinds of bomb: a blast bomb (which does damage and knocks back)
and an ice bomb (which turns squares into ice).

### Part 09.a: New Item Type (bombs)

So, we'll be adding two new items. We just stick a few new variants in our
`Item` type, and the compiler will tell us all the stuff we need to add to
handle it. Looks like just two things in the `lib.rs`:

```rust
fn apply_potion(potion: &Item, target: &mut Creature, rng: &mut PCG32) {
  match potion {
    Item::PotionHealth => target.hit_points = (target.hit_points + step(rng, 8)).min(30),
    Item::PotionStrength => target.damage_step += 1,
    _ => panic!("not a potion {}", potion),
  }
}

impl ::std::fmt::Display for Item {
  fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
    match self {
      Item::PotionHealth => write!(f, "Potion of Restore Health"),
      Item::PotionStrength => write!(f, "Potion of Gain Strength"),
      Item::BombBlast => write!(f, "Blast Bomb"),
      Item::BombIce => write!(f, "Ice Bomb"),
    }
  }
}
```

and one thing in `bin/kasidin.rs`

```rust
          Some(Item::PotionHealth) => (POTION_GLYPH, rgb32!(250, 5, 5)),
          Some(Item::PotionStrength) => (POTION_GLYPH, rgb32!(5, 240, 20)),
          Some(Item::BombBlast) => (BOMB_GLYPH, rgb32!(127, 127, 127)),
          Some(Item::BombIce) => (BOMB_GLYPH, rgb32!(153, 217, 234)),
          None => match game.terrain.get(&loc_for_this_screen_position) {
            Some(Terrain::Wall) => (WALL_TILE, rgb32!(155, 75, 0)),
            Some(Terrain::Floor) => (b'.', rgb32!(128, 128, 128)),
            None => (b' ', 0),
          },
```

And we add a pile of bombs all around the map for the player to find.

```rust
    // add some items
    for _ in 0..100 {
      let item_spot = out.pick_random_floor();
      let new_item = match out.gen.next_u32() >> 30 {
        0 => Item::PotionHealth,
        1 => Item::PotionStrength,
        2 => Item::BombBlast,
        3 => Item::BombIce,
        _ => unreachable!(),
      };
      out.item_locations.entry(item_spot).or_insert(Vec::new()).push(new_item);
    }
```

And now you can hold bombs.

![bomb-inventory](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week05-05.png)

And they look kinda neat out on the field too. When I picked out a bomb glyph I
also noticed a better potion glyph, which you can also see here.

![bomb-game](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week05-06.png)

Except if you try to "use" a bomb you get a panic because right now the item use
code assumes that you only have potions:

```
thread 'main' panicked at 'not a potion Blast Bomb', src\lib.rs:40:10
note: Run with `RUST_BACKTRACE=1` for a backtrace.
error: process didn't exit successfully: `target\debug\kasidin.exe` (exit code: 101)
```

### Part 09.b: Throwing a bomb

Okay, so we're holding bombs, now we need to throw a bomb.

What's that user experience like? Well, I _think_ that it's something like you
press `i`, then `a` (or whatever inventory letter) and then you go into a
targeting mode where you move an indicator, and then you press `enter` to
confirm or `escape` to cancel the selection.

Just one problem, we tell the game that we wanna use item `letter` and then it
gives us a `bool` for if the item was used. There's no place to pass the
targeting data! What do we do? Let's re-examine `use_item`

```rust
// impl GameWorld in lib.rs
  pub fn use_item(&mut self, item_letter: char) -> bool {
    let player_mut = self.creature_list.iter_mut().find(|creature_ref| creature_ref.is_the_player).unwrap();
    let item_to_use = {
      let mut cataloged_inventory = BTreeMap::new();
      for item_ref in player_mut.inventory.iter() {
        *cataloged_inventory.entry(item_ref).or_insert(0) += 1;
      }
      let letter_index = item_letter as u8 - 'a' as u8;
      cataloged_inventory.into_iter().nth(letter_index as usize).map(|(&item, _count)| item)
    };
    match item_to_use {
      Some(item) => {
        apply_potion(&item, player_mut, &mut self.gen);
        for i in 0..player_mut.inventory.len() {
          if player_mut.inventory[i] == item {
            player_mut.inventory.remove(i);
            break;
          }
        }
        true
      }
      None => false,
    }
  }
```

Well _frill me right into the uncharted territories_, we don't even have the
game running a turn once an item is used at the moment. That's dumb.

```rust
        for i in 0..player_mut.inventory.len() {
          if player_mut.inventory[i] == item {
            player_mut.inventory.remove(i);
            break;
          }
        }
        self.run_world_turn(); // oh my!
        true
```

There we go.

Okay so what do we do about the targeting? Well, right now that `bool` is
_kinda_ supposed to be returning if you should stay on the inventory screen or
not. Right? Here's where we use it.

```rust
// bin/kasidin.rs
        DisplayMode::Inventory => match key {
          VirtualKeyCode::Escape => display_mode = DisplayMode::Game,
          other => {
            letter_of(other).map(|ch| {
              if ch.is_alphabetic() && game.use_item(ch) {
                display_mode = DisplayMode::Game;
              }
            });
          }
        },
```

So yeah, once a letter is selected we call `use_item` and if that's true then we
switch the display mode to Game. All we gotta do is give back a better type with
more info to it.

```rust
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum UseItemResult {
  ItemUsed,
  ItemNeedsTarget,
  NoSuchItem,
}
```

And then we make use of this new type

```rust
  pub fn use_item(&mut self, item_letter: char) -> UseItemResult {
    let player_mut = self.creature_list.iter_mut().find(|creature_ref| creature_ref.is_the_player).unwrap();
    let item_to_use = {
      let mut cataloged_inventory = BTreeMap::new();
      for item_ref in player_mut.inventory.iter() {
        *cataloged_inventory.entry(item_ref).or_insert(0) += 1;
      }
      let letter_index = item_letter as u8 - 'a' as u8;
      cataloged_inventory.into_iter().nth(letter_index as usize).map(|(&item, _count)| item)
    };
    match item_to_use {
      Some(item) => {
        if item.is_potion() {
          apply_potion(&item, player_mut, &mut self.gen);
          for i in 0..player_mut.inventory.len() {
            if player_mut.inventory[i] == item {
              player_mut.inventory.remove(i);
              break;
            }
          }
          self.run_world_turn();
          UseItemResult::ItemUsed
        } else {
          UseItemResult::ItemNeedsTarget
        }
      }
      None => UseItemResult::NoSuchItem,
    }
  }
```

So then if we get `NoSuchItem` back we want to transition to the targeting
screen instead. However, unlike previous DisplayMode values that don't have any
bonus info, we also store the letter that we've got input so far, and a delta
for how offset from the player the current targeting is.

```rust
        DisplayMode::Inventory => match key {
          VirtualKeyCode::Escape => display_mode = DisplayMode::Game,
          other => {
            letter_of(other).map(|ch| {
              if ch.is_alphabetic() {
                match game.use_item(ch) {
                  UseItemResult::NoSuchItem => {}
                  UseItemResult::ItemUsed => {
                    display_mode = DisplayMode::Game;
                  }
                  UseItemResult::ItemNeedsTarget => {
                    display_mode = DisplayMode::ItemTargeting(ch, Location { x: 0, y: 0 });
                  }
                }
              }
            });
          }
        },
```

And we add key some initial key handling for the new display mode

```rust
        DisplayMode::ItemTargeting(letter, delta) => match key {
          VirtualKeyCode::Escape => display_mode = DisplayMode::Game,
          VirtualKeyCode::Up | VirtualKeyCode::Down | VirtualKeyCode::Left | VirtualKeyCode::Right => {
            let delta_change = match key {
              VirtualKeyCode::Up => Location { x: 0, y: 1 },
              VirtualKeyCode::Down => Location { x: 0, y: -1 },
              VirtualKeyCode::Left => Location { x: -1, y: 0 },
              VirtualKeyCode::Right => Location { x: 1, y: 0 },
              _ => unreachable!(),
            };
            let new_delta = delta + delta_change;
            if seen_set.contains(&(game.player_location + new_delta)) {
              display_mode = DisplayMode::ItemTargeting(letter, new_delta);
            }
          }
          _ => {}
        },
```

We'll move around the `seen_set` to accommodate this, but basically we're saying
that you can only target a square if it's currently visible to you.

The drawing for targeting mode is actually quite uninteresting. We just steal
the part form the Inventory where we draw a message at the top, and then we
steal the part from the normal game display where we draw all sorts of stuff in
everything-but-the-top. And we also track where the targeted square is and have
one little line at the end.

```rust
    if loc_for_this_screen_position == target_delta_location {
      const FULL_ALPHA: u32 = rgba32!(0, 0, 0, 255);
      fgs[(scr_x, scr_y)] = !fgs[(scr_x, scr_y)] | FULL_ALPHA;
      bgs[(scr_x, scr_y)] = !bgs[(scr_x, scr_y)] | FULL_ALPHA;
    }
```

This inverts the color of the targeted cell, so whatever the outline is we'll
still see the right shape, but "highlighted" compared to normal.

![selecting-a-target](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week05-07.png)

now we need to actually _make the bombs do something_ when you press enter.

```rust
        DisplayMode::ItemTargeting(letter, delta) => match key {
          VirtualKeyCode::Escape => display_mode = DisplayMode::Game,
          VirtualKeyCode::Return => {
            game.use_targeted_item(letter, delta);
            display_mode = DisplayMode::Game;
          }
          // ...
```

Then `use_targeted_item` does the same setup as `use_item`, and the interesting
part is where the bomb effects go into place.

The Blast Bomb does a strong range 2 blast.

```rust
      Some(Item::BombBlast) => {
        let mut blast_locations = vec![];
        let blast_center = self.player_location + target_delta;
        ppfov(
          (blast_center.x, blast_center.y),
          2,
          |x, y| self.terrain[&Location { x, y }] == Terrain::Wall,
          |x, y| blast_locations.push(Location { x, y }),
        );
        let blast_targets: Vec<CreatureID> = blast_locations
          .into_iter()
          .filter_map(|loc| self.creature_locations.get(&loc).map(|cid_ref| CreatureID(cid_ref.0)))
          .collect();
        for creature_mut in self.creature_list.iter_mut() {
          if blast_targets.contains(&creature_mut.id) {
            creature_mut.hit_points -= step(&mut self.gen, 10);
          }
        }
      }
```

And the Ice Bomb does something else interesting: it transmutes floor within
range 1 into ice. Creatures in the area (other than the player) are instantly
killed.

```rust
      Some(Item::BombIce) => {
        let mut blast_locations = vec![];
        let blast_center = self.player_location + target_delta;
        ppfov(
          (blast_center.x, blast_center.y),
          1,
          |_, _| false, /* vision check doesn't matter on radius 1 fov */
          |x, y| blast_locations.push(Location { x, y }),
        );
        for location in blast_locations.into_iter() {
          if *self.terrain.entry(location).or_insert(Terrain::Wall) == Terrain::Floor {
            *self.terrain.entry(location).or_insert(Terrain::Wall) = Terrain::Ice;
            let removed_cid = self.creature_locations.remove(&location);
            // this is a hacky way to never delete the player on accident, but
            // not really any _more_ hacky than the rest of the codebase.
            removed_cid.map(|cid_ref| {
              if cid_ref.0 > 1 {
                self.creature_list.retain(|creature_ref| &creature_ref.id != &cid_ref);
              } else {
                self.creature_locations.insert(location, CreatureID(cid_ref.0));
              }
            });
          }
        }
      }
```

And, ya know, just for more fun, we'll make blast bombs shatter the ice too.

```rust
        let mut blast_targets = vec![];
        for location in blast_locations.into_iter() {
          if *self.terrain.entry(location).or_insert(Terrain::Wall) == Terrain::Ice {
            *self.terrain.entry(location).or_insert(Terrain::Wall) = Terrain::Floor;
          }
          match self.creature_locations.get(&location) {
            None => {}
            Some(cid_ref) => {
              blast_targets.push(CreatureID(cid_ref.0));
            }
          }
        }
```

![ice-and-bombs](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week05-08.png)

The last thing to change is that Ice should probably block FOV. We have to
update once in the monster FOV code.

```rust
            ppfov(
              (loc.x, loc.y),
              7,
              |x, y| {
                let here = *terrain_ref.get(&Location { x, y }).unwrap_or(&Terrain::Wall);
                here == Terrain::Wall || here == Terrain::Ice
              },
              |x, y| {
                seen_locations.insert(Location { x, y });
              },
            );
```

And once in the player's FOV code.

```rust
    ppfov(
      (game.player_location.x, game.player_location.y),
      FOV_DISPLAY_RANGE,
      |x, y| {
        game
          .terrain
          .get(&Location { x, y })
          .map(|&t| t == Terrain::Wall || t == Terrain::Ice)
          .unwrap_or(true)
      },
      |x, y| drop(seen_set.insert(Location { x, y })),
    );
```

Messy, but it works.

During testing I noticed another bug of sorts though, we forgot items when
making the ice!

```rust
            self.item_locations.entry(location).or_insert(Vec::new()).clear();
```

We also didn't remove the appropriate item from the player's inventory when a
bomb is used. This requires a little jiggering to get all the lifetimes around,
but it's nothing a dummy scope or two can't fix!

Anyway, that's it for this week.
