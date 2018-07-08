use super::*;

pub type Path = Vec<Location>;

fn reconstruct_path(came_from: HashMap<Location, Location>, mut current: Location) -> Path {
  let mut total_path = vec![current];
  while came_from.contains_key(&current) {
    current = came_from[&current];
    total_path.push(current);
  }
  total_path
}

/// Gives the **Reverse Order** path from `start` to `end`, if any.
pub fn a_star<W>(start: Location, goal: Location, walkable: W) -> Option<Path>
where
  W: Fn(Location) -> bool,
{
  let mut closed_set = HashSet::new();
  let mut open_set = HashSet::new();
  open_set.insert(start);
  let mut came_from = HashMap::new();
  let mut g_score = HashMap::new();
  g_score.insert(start, 0i32);
  let heuristic_cost_estimate = |a: Location, b: Location| (a.x - b.x).abs().min((a.y - b.y).abs());
  let mut f_score = HashMap::new();
  f_score.insert(start, heuristic_cost_estimate(start, goal));
  while !open_set.is_empty() {
    let current = *open_set
      .iter()
      .min_by_key(|loc_ref| f_score[loc_ref])
      .expect("the open set should not have been empty because of the loop condition.");
    if current == goal {
      return Some(reconstruct_path(came_from, current));
    } else {
      open_set.remove(&current);
      closed_set.insert(current);
      for neighbor in current.neighbors().filter(|loc_ref| walkable(*loc_ref) && !closed_set.contains(loc_ref)) {
        open_set.insert(neighbor);
        let tentative_g_score = g_score[&current].saturating_add(1);
        if tentative_g_score >= *g_score.entry(neighbor).or_insert(::std::i32::MAX) {
          continue;
        } else {
          came_from.insert(neighbor, current);
          g_score.insert(neighbor, tentative_g_score);
          f_score.insert(neighbor, g_score[&neighbor] + heuristic_cost_estimate(neighbor, goal));
        }
      }
    }
  }
  None
}
