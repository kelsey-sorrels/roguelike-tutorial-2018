#![allow(unused_imports)]
#![allow(unused_mut)]

extern crate dwarf_term;
pub(crate) use dwarf_term::*;

// std
pub(crate) use std::collections::hash_map::*;
pub(crate) use std::ops::*;

pub const TILE_GRID_WIDTH: usize = 66;
pub const TILE_GRID_HEIGHT: usize = 50;

pub const WALL_TILE: u8 = 13 * 16 + 11;

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

fn make_cellular_caves(width: usize, height: usize, gen: &mut PCG32) -> VecImage<bool> {
  let d100 = RandRangeInclusive32::new(1..=100);
  let mut buffer_a: VecImage<bool> = VecImage::new(width, height);
  let mut buffer_b: VecImage<bool> = VecImage::new(width, height);
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
  // the final work is now in B
  buffer_b
}

#[derive(Debug, Clone, Default)]
pub struct GameWorld {
  pub player_location: Location,
  pub creatures: HashMap<Location, Creature>,
  pub terrain: HashMap<Location, Terrain>,
  pub gen: PCG32,
}

impl GameWorld {
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
