//! The main program!

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_imports)]
#![allow(unused_mut)]
#![allow(unused_variables)]
#![allow(dead_code)]

extern crate dwarf_term;
pub use dwarf_term::*;

extern crate roguelike_tutorial_2018;
use roguelike_tutorial_2018::*;

// std
use std::collections::hash_map::*;
use std::ops::*;

const TILE_GRID_WIDTH: usize = 66;
const TILE_GRID_HEIGHT: usize = 50;

fn main() {
  let mut term = unsafe { DwarfTerm::new(TILE_GRID_WIDTH, TILE_GRID_HEIGHT, "Kasidin").expect("WHOOPS!") };
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
      let offset = game.player_location - Location {
        x: (fgs.width() / 2) as i32,
        y: (fgs.height() / 2) as i32,
      };
      for (scr_x, scr_y, id_mut) in ids.iter_mut() {
        let loc_for_this_screen_position = Location {
          x: scr_x as i32,
          y: scr_y as i32,
        } + offset;
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
