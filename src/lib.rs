#![feature(nll)]
#![allow(unused_mut)]

extern crate dwarf_term;
pub(crate) use dwarf_term::*;

// std
pub(crate) use std::collections::hash_map::*;
pub(crate) use std::collections::hash_set::*;
pub(crate) use std::ops::*;
pub(crate) use std::sync::atomic::*;

pub mod pathing;
pub use pathing::*;
pub mod precise_permissive_fov;
pub use precise_permissive_fov::*;
pub mod prng;
pub use prng::*;

pub const WALL_TILE: u8 = 13 * 16 + 11;
pub const TERULO_BROWN: u32 = rgb32!(197, 139, 5);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, Hash)]
pub struct Location {
  pub x: i32,
  pub y: i32,
}

struct LocationNeighborsIter {
  x: i32,
  y: i32,
  index: usize,
}
impl Iterator for LocationNeighborsIter {
  type Item = Location;
  fn next(&mut self) -> Option<Self::Item> {
    match self.index {
      0 => {
        self.index += 1;
        Some(Location { x: self.x + 1, y: self.y })
      }
      1 => {
        self.index += 1;
        Some(Location { x: self.x - 1, y: self.y })
      }
      2 => {
        self.index += 1;
        Some(Location { x: self.x, y: self.y + 1 })
      }
      3 => {
        self.index += 1;
        Some(Location { x: self.x, y: self.y - 1 })
      }
      _ => None,
    }
  }
}

impl Location {
  /*
  pub fn as_usize_tuple(self) -> (usize, usize) {
    (self.x as usize, self.y as usize)
  }
  pub fn as_i32_tuple(self) -> (i32, i32) {
    (self.x, self.y)
  }
  */
  pub fn neighbors(&self) -> impl Iterator<Item = Location> {
    LocationNeighborsIter {
      x: self.x,
      y: self.y,
      index: 0,
    }
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

impl Sub for Location {
  type Output = Self;
  fn sub(self, other: Self) -> Self {
    Location {
      x: self.x - other.x,
      y: self.y - other.y,
    }
  }
}

#[derive(Debug)]
pub struct Creature {
  pub icon: u8,
  pub color: u32,
  pub is_the_player: bool,
  pub id: CreatureID,
  pub hit_points: i32,
  pub damage_step: i32,
}
impl Creature {
  fn new(icon: u8, color: u32) -> Self {
    Creature {
      icon,
      color,
      is_the_player: false,
      id: CreatureID::atomic_new(),
      hit_points: 10,
      damage_step: 5,
    }
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Terrain {
  Wall,
  Floor,
}

impl Default for Terrain {
  fn default() -> Self {
    Terrain::Wall
  }
}

fn make_cellular_caves(width: usize, height: usize, gen: &mut PCG32) -> VecImage<bool> {
  // utilities
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
  let cave_copy = |src: &VecImage<bool>, dest: &mut VecImage<bool>| {
    for (x, y, mut_ref) in dest.iter_mut() {
      // TODO: this will count up some of the cells more than once, perhaps we
      // can make this more efficient by making it more fiddly.
      *mut_ref = range_count(src, x, y, 1) >= 5 || range_count(src, x, y, 2) <= 1;
    }
  };
  let flood_copy = |src: &VecImage<bool>, dest: &mut VecImage<bool>, gen: &mut PCG32| {
    dest.set_all(true);
    let mut copied_count = 0;
    let start = {
      let d_width = RandRangeInclusive32::new(0..=((width - 1) as u32));
      let d_height = RandRangeInclusive32::new(0..=((height - 1) as u32));
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
    }
    copied_count
  };

  let d100 = RandRangeInclusive32::new(1..=100);
  let mut buffer_a: VecImage<bool> = VecImage::new(width, height);
  let mut buffer_b: VecImage<bool> = VecImage::new(width, height);

  'work: loop {
    // fill the initial buffer, all cells 45% likely.
    for (_x, _y, mut_ref) in buffer_a.iter_mut() {
      *mut_ref = d100.roll_with(gen) <= 45;
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
}

// we're setting aside '0' for a "null" type value, so the initial next value
// starts at 1.
static NEXT_CREATURE_ID: AtomicUsize = AtomicUsize::new(1);

#[derive(Debug, PartialEq, Eq, Hash)]
pub struct CreatureID(pub usize);

impl CreatureID {
  fn atomic_new() -> Self {
    CreatureID(NEXT_CREATURE_ID.fetch_add(1, Ordering::SeqCst))
  }
}

#[derive(Debug, Default)]
pub struct GameWorld {
  pub player_location: Location,
  pub creature_list: Vec<Creature>,
  pub creature_locations: HashMap<Location, CreatureID>,
  pub terrain: HashMap<Location, Terrain>,
  pub gen: PCG32,
}

impl GameWorld {
  pub fn new(seed: u64) -> Self {
    let mut out = Self {
      player_location: Location { x: 5, y: 5 },
      creature_list: vec![],
      creature_locations: HashMap::new(),
      terrain: HashMap::new(),
      gen: PCG32::new(seed),
    };
    let caves = make_cellular_caves(100, 100, &mut out.gen);
    for (x, y, tile) in caves.iter() {
      out
        .terrain
        .insert(Location { x: x as i32, y: y as i32 }, if *tile { Terrain::Wall } else { Terrain::Floor });
    }

    let mut player = Creature::new(b'@', TERULO_BROWN);
    player.is_the_player = true;
    let player_start = out.pick_random_floor();
    let player_id = player.id.0;
    out.creature_list.push(player);
    out.creature_locations.insert(player_start, CreatureID(player_id));
    out.player_location = player_start;

    for _ in 0..50 {
      let monster = Creature::new(b'k', rgb32!(166, 0, 0));
      let monster_id = monster.id.0;
      let monster_start = out.pick_random_floor();
      match out.creature_locations.entry(monster_start) {
        Entry::Occupied(_) => {
          // if we happen to pick an occupied location, just don't add a
          // creature for this pass of the loop.
          continue;
        }
        Entry::Vacant(ve) => {
          out.creature_list.push(monster);
          ve.insert(CreatureID(monster_id));
        }
      }
    }

    out
  }

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

  pub fn move_player(&mut self, delta: Location) {
    let player_move_target = self.player_location + delta;
    match self.creature_locations.get(&player_move_target) {
      Some(target_id_ref) => {
        // someone is there, do the attack!
        let player_damage_roll = {
          let player_id_ref = self.creature_locations.get(&self.player_location).unwrap();
          let player_ref = self.creature_list.iter().find(|creature_ref| &creature_ref.id == player_id_ref).unwrap();
          step4(&mut self.gen, player_ref.damage_step)
        };
        let target_ref_mut = self
          .creature_list
          .iter_mut()
          .find(|creature_mut_ref| &creature_mut_ref.id == target_id_ref)
          .unwrap();
        target_ref_mut.hit_points -= player_damage_roll;
        println!("Player did {} damage to {:?}", player_damage_roll, target_id_ref);
      }
      None => {
        // no one is there, move
        match *self.terrain.entry(player_move_target).or_insert(Terrain::Floor) {
          Terrain::Wall => {
            // Accidentally bumping a wall doesn't consume a turn.
            return;
          }
          Terrain::Floor => {
            let player_id = self
              .creature_locations
              .remove(&self.player_location)
              .expect("The player wasn't where they should be!");
            let old_creature = self.creature_locations.insert(player_move_target, player_id);
            debug_assert!(old_creature.is_none());
            self.player_location = player_move_target;
          }
        }
      }
    }
    self.run_world_turn();
    println!("turn over!");
  }

  pub fn run_world_turn(&mut self) {
    let initiative_list: Vec<CreatureID> = self
      .creature_list
      .iter()
      .filter_map(|creature_mut| {
        if creature_mut.is_the_player || creature_mut.hit_points < 1 {
          None
        } else {
          Some(CreatureID(creature_mut.id.0))
        }
      })
      .collect();
    for creature_id_ref in initiative_list.iter() {
      let my_location: Option<Location> = {
        self
          .creature_locations
          .iter()
          .find(|&(_loc, id)| id == creature_id_ref)
          .map(|(&loc, _id)| loc)
      };
      match my_location {
        None => println!("Creature {:?} is not anywhere!", creature_id_ref),
        Some(loc) => {
          // Look around
          let seen_locations = {
            let terrain_ref = &self.terrain;
            let mut seen_locations = HashSet::new();
            ppfov(
              (loc.x, loc.y),
              7,
              |x, y| terrain_ref.get(&Location { x, y }).unwrap_or(&Terrain::Wall) == &Terrain::Wall,
              |x, y| {
                seen_locations.insert(Location { x, y });
              },
            );
            seen_locations
          };
          // Decide where to go
          let move_target = if seen_locations.contains(&self.player_location) {
            let terrain_ref = &self.terrain;
            let path = a_star(self.player_location, loc, |loc| {
              terrain_ref.get(&loc).unwrap_or(&Terrain::Wall) != &Terrain::Wall
            }).expect("couldn't find a path");
            debug_assert_eq!(loc, path[0]);
            path[1]
          } else {
            loc + match self.gen.next_u32() >> 30 {
              0 => Location { x: 0, y: 1 },
              1 => Location { x: 0, y: -1 },
              2 => Location { x: 1, y: 0 },
              3 => Location { x: -1, y: 0 },
              impossible => unreachable!("u32 >> 30: {}", impossible),
            }
          };
          // go there
          match self.creature_locations.get(&move_target) {
            Some(target_id_ref) => {
              // someone is there, do the attack!
              let creature_damage_roll = {
                let creature_ref = self
                  .creature_list
                  .iter()
                  .find(|creature_ref| &creature_ref.id == creature_id_ref)
                  .unwrap();
                step4(&mut self.gen, creature_ref.damage_step)
              };
              let target_ref_mut = self
                .creature_list
                .iter_mut()
                .find(|creature_mut_ref| &creature_mut_ref.id == target_id_ref)
                .unwrap();
              if target_ref_mut.is_the_player {
                target_ref_mut.hit_points -= creature_damage_roll;
                println!("{:?} did {} damage to {:?}", creature_id_ref, creature_damage_roll, target_id_ref);
              }
              // TODO: log that we did damage.
            }
            None => match *self.terrain.entry(move_target).or_insert(Terrain::Floor) {
              Terrain::Wall => {
                continue;
              }
              Terrain::Floor => {
                let id = self.creature_locations.remove(&loc).expect("The creature wasn't where they should be!");
                let old_id = self.creature_locations.insert(move_target, id);
                debug_assert!(old_id.is_none());
              }
            },
          }
        }
      }
    }
    // End Phase, we clear any dead NPCs off the list.
    let creature_locations_mut = &mut self.creature_locations;
    self.creature_list.retain(|creature_ref| {
      let keep = creature_ref.hit_points > 0 || creature_ref.is_the_player;
      if !keep {
        let dead_location = *creature_locations_mut
          .iter()
          .find(|&(_, v_cid)| v_cid == &creature_ref.id)
          .expect("Locations list out of sync!")
          .0;
        creature_locations_mut.remove(&dead_location);
      };
      keep
    });
  }
}
