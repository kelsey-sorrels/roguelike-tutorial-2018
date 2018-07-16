//! The main program!

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![allow(unused_mut)]

extern crate dwarf_term;
pub use dwarf_term::*;

extern crate roguelike_tutorial_2018;
use roguelike_tutorial_2018::*;

// std
use std::collections::btree_map::*;
use std::collections::hash_set::*;
use std::io::*;

const TILE_GRID_WIDTH: usize = 66;
const TILE_GRID_HEIGHT: usize = 50;
const KINDA_LIME_GREEN: u32 = rgb32!(128, 255, 20);

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum DisplayMode {
  Game,
  Inventory,
}

fn main() {
  let mut term = unsafe { DwarfTerm::new(TILE_GRID_WIDTH, TILE_GRID_HEIGHT, "Kasidin").expect("WHOOPS!") };
  term.set_all_foregrounds(KINDA_LIME_GREEN);
  term.set_all_backgrounds(0);

  let mut game = GameWorld::new(u64_from_time());

  // Main loop
  let mut running = true;
  let mut pending_keys = vec![];
  let mut display_mode = DisplayMode::Game;
  'game: loop {
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
    if !running {
      break 'game;
    }

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
          other => {
            letter_of(other).map(|ch| {
              if ch.is_alphabetic() && game.use_item(ch) {
                display_mode = DisplayMode::Game;
              }
            });
          }
        },
      }
    }

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

fn draw_game(term: &mut DwarfTerm, game: &GameWorld, seen_set: &HashSet<(i32, i32)>) {
  let (mut fgs, mut bgs, mut ids) = term.layer_slices_mut();
  // we don't clear the display since we'll write to all of the display

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
  fgs.slice_mut((0, map_view_end.1)..full_extent).set_all(KINDA_LIME_GREEN);
  bgs.slice_mut((0, map_view_end.1)..full_extent).set_all(rgb32!(0, 0, 0));
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

fn draw_inventory(term: &mut DwarfTerm, game: &GameWorld) {
  let (mut fgs, mut bgs, mut ids) = term.layer_slices_mut();
  // clear the display
  fgs.set_all(rgb32!(255, 255, 255));
  bgs.set_all(rgb32!(0, 0, 0));
  ids.set_all(0);

  let mut map_item_count = BTreeMap::new();
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

  let mut item_list = vec![];
  for (key, val) in map_item_count.into_iter() {
    match val {
      0 => panic!("what the heck?"),
      1 => item_list.push(format!("{}", key)),
      count => item_list.push(format!("{} ({})", key, count)),
    }
  }

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
      write!(this_line_slice_mut, "{}) {}", letter as u8 as char, item).ok();
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
}
