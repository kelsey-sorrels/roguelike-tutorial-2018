#![allow(unused_mut)]

extern crate dwarf_term;
pub(crate) use dwarf_term::*;

// std
pub(crate) use std::collections::hash_map::*;
pub(crate) use std::collections::hash_set::*;
pub(crate) use std::ops::*;
pub(crate) use std::sync::atomic::*;

pub mod precise_permissive_fov;
pub use precise_permissive_fov::*;

pub const WALL_TILE: u8 = 13 * 16 + 11;
pub const TERULO_BROWN: u32 = rgb32!(197, 139, 5);

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, Hash)]
pub struct Location {
  pub x: i32,
  pub y: i32,
}

impl Location {
  pub fn as_usize(self) -> (usize, usize) {
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
}
impl Creature {
  fn new(icon: u8, color: u32) -> Self {
    Creature {
      icon,
      color,
      is_the_player: false,
      id: CreatureID::atomic_new(),
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
      gen: PCG32 { state: seed },
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
    if self.creature_locations.contains_key(&player_move_target) {
      println!("Player does a bump!");
    } else {
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
    self.run_world_turn();
  }

  pub fn run_world_turn(&mut self) {
    for creature_mut in self.creature_list.iter_mut() {
      if creature_mut.is_the_player {
        continue;
      } else {
        let my_location: Option<Location> = {
          self
            .creature_locations
            .iter()
            .find(|&(&_loc, id)| id == &creature_mut.id)
            .map(|(&loc, _id)| loc)
        };
        match my_location {
          None => println!("Creature {:?} is not anywhere!", creature_mut.id),
          Some(loc) => {
            let move_target = loc + match self.gen.next_u32() >> 30 {
              0 => Location { x: 0, y: 1 },
              1 => Location { x: 0, y: -1 },
              2 => Location { x: 1, y: 0 },
              3 => Location { x: -1, y: 0 },
              impossible => unreachable!("u32 >> 30: {}", impossible),
            };
            if self.creature_locations.contains_key(&move_target) {
              println!("{:?} does a bump!", creature_mut.id);
            } else {
              match *self.terrain.entry(move_target).or_insert(Terrain::Floor) {
                Terrain::Wall => {
                  continue;
                }
                Terrain::Floor => {
                  let id = self.creature_locations.remove(&loc).expect("The creature wasn't where they should be!");
                  let old_id = self.creature_locations.insert(move_target, id);
                  debug_assert!(old_id.is_none());
                }
              }
            }
          }
        }
      }
    }
  }
}

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

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RandRangeInclusive32 {
  base: u32,
  width: u32,
  reject: u32,
}

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

  pub fn roll_with(&self, gen: &mut PCG32) -> u32 {
    loop {
      if let Some(output) = self.convert(gen.next_u32()) {
        return output;
      }
    }
  }
}

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
