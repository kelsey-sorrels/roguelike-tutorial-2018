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

TODO: random item placement

TODO: Items shown in the camera

### Place 08.b: Auto-pickup

TODO: Coding

### Place 08.c: Consuming Items

TODO: Coding

### Place 08.d: Dropping Items

TODO: Coding

### Place 08.e: Equipping Items

Ha, you thought that we'd do all the basic item stuff at the same time? How foolish!

Of course we have to wait until Part 13 to actually _equip_ an item.

Doing the tutorial out of order would simply be heretical.

## Part 09: Ranged Attacks and Targeting

TODO: Preview

### Part 09.a: New Item Type (bombs)

TODO: Coding

### Part 09.b: Targeting a square

TODO: Coding
