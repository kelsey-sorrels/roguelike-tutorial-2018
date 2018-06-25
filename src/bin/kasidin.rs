//! The main program!

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(dead_code)]

extern crate dwarf_term;
pub use dwarf_term::*;

// std
use std::collections::hash_map::*;
use std::ops::*;

const TILE_GRID_WIDTH: usize = 66;
const TILE_GRID_HEIGHT: usize = 50;

const WALL_TILE: u8 = 13 * 16 + 11;

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

fn make_cellular_caves(width: usize, height: usize, gen: &mut PCG32) -> VecImage<bool> {
  unimplemented!()
}

#[derive(Debug, Clone, Default)]
struct GameWorld {
  player_location: Location,
  creatures: HashMap<Location, Creature>,
  terrain: HashMap<Location, Terrain>,
  gen: PCG32,
}

impl GameWorld {
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

fn main() {
  let mut term = unsafe { DwarfTerm::new(TILE_GRID_WIDTH, TILE_GRID_HEIGHT, "Kasidin Test").expect("WHOOPS!") };
  term.set_all_foregrounds(rgb32!(128, 255, 20));
  term.set_all_backgrounds(0);

  let mut game = GameWorld::new(u64_from_time());

  // Main loop
  let mut running = true;
  let mut pending_keys = vec![];
  'game: loop {
    // Grab all new presses
    term.poll_events(|event| match event {
      Event::WindowEvent { event: win_event, .. } => match win_event {
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
    if !running {
      // TODO: Escape should not kill the game instantly in the final program
      break 'game;
    }

    for key in pending_keys.drain(..) {
      match key {
        VirtualKeyCode::Up => game.move_player(Location { x: 0, y: 1 }),
        VirtualKeyCode::Down => game.move_player(Location { x: 0, y: -1 }),
        VirtualKeyCode::Left => game.move_player(Location { x: -1, y: 0 }),
        VirtualKeyCode::Right => game.move_player(Location { x: 1, y: 0 }),
        _ => {}
      }
    }

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

    unsafe {
      term
        .clear_draw_swap()
        .map_err(|err_vec| {
          for e in err_vec {
            eprintln!("clear_draw_swap error: {:?}", e);
          }
        })
        .ok();
    }
  }
}
