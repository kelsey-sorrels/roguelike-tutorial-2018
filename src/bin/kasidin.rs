//! The main program!

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_mut)]

extern crate dwarf_term;
pub use dwarf_term::*;

extern crate roguelike_tutorial_2018;
use roguelike_tutorial_2018::*;

// std
use std::collections::hash_set::*;
use std::io::*;

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

    let mut seen_set = HashSet::new();
    ppfov(
      (game.player_location.x, game.player_location.y),
      25,
      |x, y| game.terrain.get(&Location { x, y }).map(|&t| t == Terrain::Wall).unwrap_or(true),
      |x, y| {
        seen_set.insert((x, y));
      },
    );
    {
      let (mut fgs, mut _bgs, mut ids) = term.layer_slices_mut();
      let offset = game.player_location - Location {
        x: (fgs.width() / 2) as i32,
        y: (fgs.height() / 2) as i32,
      };
      // draw the map, save space for the status line.
      const STATUS_HEIGHT: usize = 1;
      let full_extent = (ids.width(), ids.height());
      let map_view_end = (full_extent.0, full_extent.1 - STATUS_HEIGHT);
      for (scr_x, scr_y, id_mut) in ids.slice_mut((0, 0)..map_view_end).iter_mut() {
        let loc_for_this_screen_position = Location {
          x: scr_x as i32,
          y: scr_y as i32,
        } + offset;
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
      }
      // draw the status bar.
      let mut ids_status_slice_mut = ids.slice_mut((0, map_view_end.1)..full_extent);
      debug_assert_eq!(ids_status_slice_mut.width(), full_extent.0);
      debug_assert_eq!(ids_status_slice_mut.height(), STATUS_HEIGHT);
      ids_status_slice_mut.set_all(0);
      debug_assert_eq!(1, STATUS_HEIGHT);
      let mut status_line_u8_slice_mut: &mut [u8] = unsafe { ::std::slice::from_raw_parts_mut(ids_status_slice_mut.as_mut_ptr(), full_extent.0) };
      let player_hp = game
        .creature_list
        .iter()
        .find(|creature_ref| creature_ref.is_the_player)
        .unwrap()
        .hit_points;
      write!(status_line_u8_slice_mut, "HP: {}, Enemies: {}", player_hp, game.creature_list.len() - 1).ok();
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
