//! Does precise permissive FOV calculations.

use std::collections::hash_set::*;

#[derive(Debug, Clone, Copy)]
struct Line {
  xi: i32,
  yi: i32,
  xf: i32,
  yf: i32,
}
impl Line {
  fn new(xi: i32, yi: i32, xf: i32, yf: i32) -> Self {
    Self { xi, yi, xf, yf }
  }
  fn dx(&self) -> i32 {
    self.xf - self.xi
  }
  fn dy(&self) -> i32 {
    self.yf - self.yi
  }
  fn relative_slope(&self, x: i32, y: i32) -> i32 {
    (self.dy() * (self.xf - x)) - (self.dx() * (self.yf - y))
  }
  fn below_p(&self, x: i32, y: i32) -> bool {
    self.relative_slope(x, y) > 0
  }
  fn below_or_collinear_p(&self, x: i32, y: i32) -> bool {
    self.relative_slope(x, y) >= 0
  }
  fn above_p(&self, x: i32, y: i32) -> bool {
    self.relative_slope(x, y) < 0
  }
  fn above_or_collinear_p(&self, x: i32, y: i32) -> bool {
    self.relative_slope(x, y) <= 0
  }
  fn collinear_p(&self, x: i32, y: i32) -> bool {
    self.relative_slope(x, y) == 0
  }
  fn collinear_line(&self, line: Line) -> bool {
    self.collinear_p(line.xi, line.yi) && self.collinear_p(line.xf, line.yf)
  }
}

#[test]
fn line_tests() {
  let line_a = Line::new(0, 0, 1, 1);
  let line_b = Line::new(2, 2, 3, 3);
  assert!(line_a.collinear_line(line_b));

  assert!(line_a.collinear_p(0, 0));
  assert!(line_a.above_or_collinear_p(0, 0));
  assert!(line_a.above_p(1, 0));

  assert!(line_a.below_or_collinear_p(0, 1));
  assert!(line_a.below_p(0, 1));
}

#[derive(Debug, Clone)]
struct View {
  shallow_line: Line,
  steep_line: Line,
  shallow_bumps: Vec<(i32, i32)>,
  steep_bumps: Vec<(i32, i32)>,
}
impl View {
  fn new(shallow_line: Line, steep_line: Line) -> Self {
    Self {
      shallow_line,
      steep_line,
      shallow_bumps: vec![],
      steep_bumps: vec![],
    }
  }

  fn add_shallow_bump(&mut self, x: i32, y: i32) {
    self.shallow_line.xf = x;
    self.shallow_line.yf = y;
    self.shallow_bumps.insert(0, (x, y));
    for bump in self.steep_bumps.iter() {
      if self.shallow_line.above_p(bump.0, bump.1) {
        self.shallow_line.xi = bump.0;
        self.shallow_line.yi = bump.1;
      }
    }
  }

  fn add_steep_bump(&mut self, x: i32, y: i32) {
    self.steep_line.xf = x;
    self.steep_line.yf = y;
    self.steep_bumps.insert(0, (x, y));
    for bump in self.shallow_bumps.iter() {
      if self.steep_line.below_p(bump.0, bump.1) {
        self.steep_line.xi = bump.0;
        self.steep_line.yi = bump.1;
      }
    }
  }
}

#[test]
fn view_tests() {
  // the red line in `week03-04.png`
  let shallow_line = Line::new(0, 1, 5, 0);
  // the blue line in `week03-04.png`
  let steep_line = Line::new(1, 0, 0, 5);
  let mut the_view = View::new(shallow_line, steep_line);

  // add square 5's lower-right as a steep bump
  the_view.add_steep_bump(1, 2);

  // We should now look like `week03-05.png`, with the steep line being vertical
  // up from (1,0)
  assert!(the_view.steep_line.collinear_p(1, 1));
  assert!(the_view.shallow_line.collinear_line(shallow_line));

  // let's move to `week03-06.png`.
  // * we clone the view
  // * one view is steep bumped by 7's lower-right
  // * the other view is shallow bumped by 7's upper-left
  let mut red_blue_view = the_view.clone();
  let mut yellow_purple_view = the_view.clone();

  red_blue_view.add_steep_bump(3, 1);
  red_blue_view.shallow_line.collinear_p(3, 1);

  yellow_purple_view.add_shallow_bump(2, 2);
  yellow_purple_view.steep_line.collinear_p(2, 2);
}

fn check_quadrant<VB, VE>(
  visited: &mut HashSet<(i32, i32)>, start_x: i32, start_y: i32, dir_x: i32, dir_y: i32, extent_x: i32, extent_y: i32, vision_blocked: &VB,
  visit_effect: &mut VE,
) where
  VB: Fn(i32, i32) -> bool,
  VE: FnMut(i32, i32),
{
  debug_assert!(dir_x == -1 || dir_x == 1);
  debug_assert!(dir_y == -1 || dir_y == 1);
  debug_assert!(extent_x > 0);
  debug_assert!(extent_y > 0);

  let shallow_line = Line::new(0, 1, extent_x, 0);
  let steep_line = Line::new(1, 0, 0, extent_y);
  let mut active_views = vec![View::new(shallow_line, steep_line)];

  for i in 1..=(extent_x + extent_y) {
    for j in (i - extent_x).max(0)..=i.min(extent_y) {
      if active_views.is_empty() {
        return;
      } else {
        let offset_x = i - j;
        let offset_y = j;
        visit_coord(
          visited,
          start_x,
          start_y,
          dir_x,
          dir_y,
          vision_blocked,
          visit_effect,
          offset_x,
          offset_y,
          &mut active_views,
        );
      }
    }
  }
}

fn visit_coord<VB, VE>(
  visited: &mut HashSet<(i32, i32)>, start_x: i32, start_y: i32, dir_x: i32, dir_y: i32, vision_blocked: &VB, visit_effect: &mut VE, offset_x: i32,
  offset_y: i32, active_views: &mut Vec<View>,
) where
  VB: Fn(i32, i32) -> bool,
  VE: FnMut(i32, i32),
{
  debug_assert!(dir_x == -1 || dir_x == 1);
  debug_assert!(dir_y == -1 || dir_y == 1);
  debug_assert!(offset_x >= 0);
  debug_assert!(offset_y >= 0);

  let top_left = (offset_x, offset_y + 1);
  let bottom_right = (offset_x + 1, offset_y);
  let mut view_index = 0;
  loop {
    match active_views.get(view_index) {
      None => return,
      Some(view_ref) => if view_ref.steep_line.below_or_collinear_p(bottom_right.0, bottom_right.1) {
        view_index += 1;
      } else if view_ref.shallow_line.above_or_collinear_p(top_left.0, top_left.1) {
        return;
      } else {
        break;
      },
    }
  }

  let target = (start_x + (offset_x * dir_x), start_y + (offset_y * dir_y));
  if !visited.contains(&target) {
    visited.insert(target);
    visit_effect(target.0, target.1);
  }

  if vision_blocked(target.0, target.1) {
    match (
      active_views[view_index].shallow_line.above_p(bottom_right.0, bottom_right.1),
      active_views[view_index].steep_line.below_p(top_left.0, top_left.1),
    ) {
      (true, true) => {
        // The shallow line and steep line both intersect this location, and
        // sight is blocked here, so this view is dead.
        active_views.remove(view_index);
      }
      (true, false) => {
        // The shallow line intersects here but the steep line does not, so we
        // add this location as a shallow bump and check our views.
        active_views[view_index].add_shallow_bump(top_left.0, top_left.1);
        check_view(active_views, view_index);
      }
      (false, true) => {
        // the steep line intersects here but the shallow line does not, so we
        // add a steep bump at this location and check our views.
        active_views[view_index].add_steep_bump(bottom_right.0, bottom_right.1);
        check_view(active_views, view_index);
      }
      (false, false) => {
        // Neither line intersects this location but it blocks sight, so we have
        // to split this view into two views.
        let new_view = active_views[view_index].clone();
        active_views.insert(view_index, new_view);
        // We add the shallow bump on the farther view first, so that if it gets
        // killed we don't have to change where we add the steep bump and check
        active_views[view_index + 1].add_shallow_bump(top_left.0, top_left.1);
        check_view(active_views, view_index + 1);
        active_views[view_index].add_steep_bump(bottom_right.0, bottom_right.1);
        check_view(active_views, view_index);
      }
    }
  }
}

fn check_view(active_views: &mut Vec<View>, view_index: usize) {
  let view_is_dead = {
    let shallow_line = active_views[view_index].shallow_line;
    let steep_line = active_views[view_index].steep_line;
    shallow_line.collinear_line(steep_line) && (shallow_line.collinear_p(0, 1) || shallow_line.collinear_p(1, 0))
  };
  if view_is_dead {
    active_views.remove(view_index);
  }
}

/// Computes field of view according to the "Precise Permissive" technique.
///
/// [See the RogueBasin page](http://www.roguebasin.com/index.php?title=Precise_Permissive_Field_of_View)
pub fn ppfov<VB, VE>((start_x, start_y): (i32, i32), radius: i32, vision_blocked: VB, mut visit_effect: VE)
where
  VB: Fn(i32, i32) -> bool,
  VE: FnMut(i32, i32),
{
  debug_assert!(radius >= 0, "ppfov: vision radius must be non-negative, got {}", radius);
  debug_assert!(
    start_x.saturating_add(radius) < ::std::i32::MAX,
    "ppfov: Location ({},{}) with radius {} would cause overflow problems!",
    start_x,
    start_y,
    radius
  );
  debug_assert!(
    start_y.saturating_add(radius) < ::std::i32::MAX,
    "ppfov: Location ({},{}) with radius {} would cause overflow problems!",
    start_x,
    start_y,
    radius
  );
  debug_assert!(
    start_x.saturating_sub(radius) > ::std::i32::MIN,
    "ppfov: Location ({},{}) with radius {} would cause underflow problems!",
    start_x,
    start_y,
    radius
  );
  debug_assert!(
    start_y.saturating_sub(radius) > ::std::i32::MIN,
    "ppfov: Location ({},{}) with radius {} would cause underflow problems!",
    start_x,
    start_y,
    radius
  );

  let mut visited = HashSet::new();
  visit_effect(start_x, start_y);
  visited.insert((start_x, start_y));

  // q1
  check_quadrant(&mut visited, start_x, start_y, 1, 1, radius, radius, &vision_blocked, &mut visit_effect);
  // q2
  check_quadrant(&mut visited, start_x, start_y, -1, 1, radius, radius, &vision_blocked, &mut visit_effect);
  // q3
  check_quadrant(&mut visited, start_x, start_y, -1, -1, radius, radius, &vision_blocked, &mut visit_effect);
  // q4
  check_quadrant(&mut visited, start_x, start_y, 1, -1, radius, radius, &vision_blocked, &mut visit_effect);
}
