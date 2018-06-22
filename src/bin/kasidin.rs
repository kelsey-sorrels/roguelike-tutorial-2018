//! The main program!

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(dead_code)]

extern crate dwarf_term;
pub use dwarf_term::*;

extern crate specs;
use specs::prelude::*;

// std
use std::collections::{HashMap, HashSet};

const TILE_GRID_WIDTH: usize = 66;
const TILE_GRID_HEIGHT: usize = 50;

#[derive(Debug, PartialEq, Eq, Clone, Copy, Default, Hash)]
struct Location {
  x: i32,
  y: i32,
}

impl Component for Location {
  type Storage = DenseVecStorage<Self>;
}

#[derive(Debug, Clone, Copy, Default, Hash)]
struct Player(u8);

impl Component for Player {
  type Storage = HashMapStorage<Self>;
}

#[derive(Debug, Clone, Copy, Default, Hash)]
struct Collider;

impl Component for Collider {
  type Storage = NullStorage<Self>;
}

/// Holds a Vec of (player_id, key)
#[derive(Debug, Default)]
struct KeyPressEvents(Vec<(u8, VirtualKeyCode)>);

pub type Color = u32;
pub type Tile = u8;

#[derive(Debug, Default, Clone, Copy)]
pub struct Cell {
  pub tile: Tile,
  pub fg: Color,
  pub bg: Color,
}

/// Info on the "camera" for the game.
#[derive(Debug)]
struct CameraData {
  offset: Location,
  cells: VecImage<Cell>,
}
impl CameraData {
  fn new(width: usize, height: usize) -> Self {
    CameraData {
      offset: Location::default(),
      cells: VecImage::new(width, height),
    }
  }
}

#[derive(Debug)]
struct PlayerInputSystem;

impl<'a> System<'a> for PlayerInputSystem {
  type SystemData = (
    WriteStorage<'a, Location>,
    ReadStorage<'a, Player>,
    ReadStorage<'a, Collider>,
    Read<'a, KeyPressEvents>,
  );

  fn run(&mut self, (mut locations, players, colliders, key_press_events): Self::SystemData) {
    for (loc, player_value) in (&mut locations, &players).join() {
      for (key_id, key) in key_press_events.0.iter() {
        if *key_id != player_value.0 {
          continue;
        } else {
          match key {
            VirtualKeyCode::Up => loc.y += 1,
            VirtualKeyCode::Down => loc.y -= 1,
            VirtualKeyCode::Left => loc.x -= 1,
            VirtualKeyCode::Right => loc.x += 1,
            _ => {}
          }
        }
      }
    }
  }
}

#[derive(Debug)]
struct CameraUpdateSystem;

impl<'a> System<'a> for CameraUpdateSystem {
  type SystemData = (WriteStorage<'a, Location>, WriteExpect<'a, CameraData>);

  fn run(&mut self, (locations, mut camera_data): Self::SystemData) {
    let the_offset = camera_data.offset;
    for loc in locations.join() {
      if *loc == (Location { x: 10, y: 10 }) {
        camera_data.cells.get_mut((10, 10)).map(|mut_ref| mut_ref.tile = b'#');
        return;
      }
      // TODO: Location math ops
      // TODO: Location.as_usize() op
      let camera_position = ((loc.x + the_offset.x) as usize, (loc.y + the_offset.y) as usize);
      camera_data.cells.get_mut(camera_position).map(|mut_ref| mut_ref.tile = b'@');
    }
  }
}

fn main() {
  let mut term = unsafe { DwarfTerm::new(TILE_GRID_WIDTH, TILE_GRID_HEIGHT, "Kasidin Test").expect("WHOOPS!") };
  term.set_all_foregrounds(rgb32!(128, 255, 20));
  term.set_all_backgrounds(0);

  let mut world = World::new();
  world.register::<Location>();
  world.register::<Player>();
  world.register::<Collider>();
  world.add_resource(KeyPressEvents(vec![]));
  world.add_resource(CameraData::new(TILE_GRID_WIDTH, TILE_GRID_HEIGHT));

  let mut dispatcher = DispatcherBuilder::new()
    .with(PlayerInputSystem, "player_input", &[])
    .with(CameraUpdateSystem, "camera_update", &["player_input"])
    .build();

  // Right now there's only a single player, but theoretically we could get
  // other players I guess. Multi-player roguelikes are a chronic pipe dream.
  const THE_PLAYER: u8 = 1;

  // Kasidin starts at 5,5
  world
    .create_entity()
    .with(Location { x: 5, y: 5 })
    .with(Player(THE_PLAYER))
    .with(Collider)
    .build();

  // our wall buddy starts at 10,10
  world.create_entity().with(Location { x: 10, y: 10 }).with(Collider).build();

  // Main loop
  let mut running = true;
  'game: loop {
    // clear any "per frame" resource data
    world.write_resource::<KeyPressEvents>().0.clear();
    world.write_resource::<CameraData>().cells.set_all(Cell::default());

    // then grab all new presses
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
          world.write_resource::<KeyPressEvents>().0.push((THE_PLAYER, key));
        }
        _ => {}
      },
      _ => {}
    });
    if !running {
      // TODO: Escape should not kill the game instantly in the final program
      break 'game;
    }

    // dispatch the system!
    dispatcher.dispatch(&mut world.res);

    // Copy our camera results to the actual terminal. The `direct_copy` method
    // uses `mem_copy` internally, so it's very fast. We put it inside a dummy
    // scope so that the image slices all go away and the mutable borrow on the
    // terminal ends before it's time to call `clear_draw_swap`.
    {
      let (mut fgs, mut bgs, mut ids) = term.layer_slices_mut();
      for (x, y, cell) in world.read_resource::<CameraData>().cells.iter() {
        fgs[(x, y)] = cell.fg;
        bgs[(x, y)] = cell.bg;
        ids[(x, y)] = cell.tile;
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
