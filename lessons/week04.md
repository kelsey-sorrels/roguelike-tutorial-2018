# Week 04

## Part 06: Fighting Each Other

### Part 06a: To Battle, And Victory!

So part 6 is supposed to be about fighting, kinda, but we don't have a way for
the creatures to try and move toward their enemy yet. First we'll add a way to
compute a path for them to follow.

We're going to use stuff based on [A*
pathing](https://en.wikipedia.org/wiki/A*_search_algorithm) (pronounced
"A-star") because it's easy to write and it's the basic stuff. However, as with
the PRNG stuff from earlier, once you understand the basics of pathing and A*
you can more easily use additional techniques such as [Djikstra
maps](http://www.roguebasin.com/index.php?title=The_Incredible_Power_of_Dijkstra_Maps)
and [JPS
pathing](https://www.gdcvault.com/play/1022094/JPS-Over-100x-Faster-than). They
pre-compute more stuff ahead of time to make each path faster to compute during
the actual level's usage. The most classic of trade offs.

Thankfully, as with before, we just gotta read the wikipedia article to find out
how this works. Turns out it was invented to help a _robot friend_. This buddy:

![image](https://en.wikipedia.org/wiki/A*_search_algorithm#/media/File:SRI_Shakey_with_callouts.jpg)

Look at that pal. Adorable.

> A* is an informed search algorithm, or a best-first search,

So our monsters will have "magic" knowledge of the dungeon if we ask them to
path toward something outside their FOV. If we want to do something where we
have only partial knowledge of the map we'd probably need something outside A*.

> At each iteration of its main loop, A* needs to determine which of its partial
> paths to expand into one or more longer paths. It does so based on an estimate
> of the cost (total weight) still to go to the goal node. Specifically, A*
> selects the path that minimizes
> 
>    f(n) = g(n) + h(n)
> 
> where n is the last node on the path, g(n) is the cost of the path from the
> start node to n, and h(n) is a heuristic that estimates the cost of the
> cheapest path from n to the goal.

Oh, _good_, trust Stanford to put a bunch of useless function names into the
process. This is why math gets a bad reputation folks.

> The heuristic is problem-specific. For the algorithm to find the actual
> shortest path, the heuristic function must be admissible, meaning that it
> never overestimates the actual cost to get to the nearest goal node.

Okay. Never estimate too much. Can do.

> Typical implementations of A* use a priority queue to perform the repeated
> selection of minimum (estimated) cost nodes to expand.

Hmm, do we have that in the standard lib? [Looks like
nope](https://doc.rust-lang.org/std/index.html?search=priority). So we'll build
something for that, no problem.

> This priority queue is known as the _open set_ or _fringe_. At each step of
> the algorithm, the node with the lowest f(x) value is removed from the queue,
> the f and g values of its neighbors are updated accordingly, and these
> neighbors are added to the queue. The algorithm continues until a goal node
> has a lower f value than any node in the queue (or until the queue is empty).
> The f value of the goal is then the length of the shortest path, since h at
> the goal is zero in an admissible heuristic.

Ah, okay...

> The algorithm described so far gives us only the length of the shortest path.
> To find the actual sequence of steps, the algorithm can be easily revised so
> that each node on the path keeps track of its predecessor. After this
> algorithm is run, the ending node will point to its predecessor, and so on,
> until some node's predecessor is the start node.

Alright, post-processing step, easy to understand there.

> If the heuristic h satisfies the additional condition h(x) ≤ d(x, y) + h(y)
> for every edge (x, y) of the graph (where d denotes the length of that edge),
> then h is called monotone, or consistent. In such a case, A* can be
> implemented more efficiently - roughly speaking, no node needs to be processed
> more than once (see closed set below) - and A* is equivalent to running
> Dijkstra's algorithm with the reduced cost d'(x, y) = d(x, y) + h(y) − h(x).

We _do_ satisfy this constraint, so we will get to use the closed set
optimization.

> The following pseudocode describes the algorithm:

Oh thank Eris.

```
function reconstruct_path(cameFrom, current)
    total_path := {current}
    while current in cameFrom.Keys:
        current := cameFrom[current]
        total_path.append(current)
    return total_path
```

Hmm. Shouldn't be too hard. So, accounting for rust's silly `snake_case`
convention, we'll have `came_from`, which is some sort of `HashMap` based on it
having a set of keys and all that. We also have `current`, which is... just a
single Location I guess? It's used as the key into the `came_from` mapping, so
that gives us the key type. It's also written to as the output of the
`cameFrom[current]` expression, which gives us the value type of the mapping.
Okay, so we're set to make our function signature.

```rust
fn reconstruct_path(came_from: HashMap<Location, Location>, current: Location) -> Path {
  // ?
}
```

What's a `Path`? Well, it looks like we can just use `Vec<Location>` for now.

```rust
pub type Path = Vec<Location>;
```

And this lets us fill in a bit more of our code.

```rust
fn reconstruct_path(came_from: HashMap<Location, Location>, current: Location) -> Path {
  let mut total_path = vec![current];
  // ?
  total_path
}
```

And then just carefully place a `while` loop in there...

```rust
fn reconstruct_path(came_from: HashMap<Location, Location>, mut current: Location) -> Path {
  let mut total_path = vec![current];
  while came_from.contains_key(&current) {
    current = came_from[&current];
    total_path.push(current);
  }
  total_path
}
```

Great. Now we can reconstruct a path given a correctly populated `HashMap`.
Getting that is a little more complicated. Let's look at more pseudocode!

```
function A_Star(start, goal)
    closedSet := {}
    openSet := {start}
    cameFrom := an empty map

    gScore := map with default value of Infinity
    gScore[start] := 0

    fScore := map with default value of Infinity
    fScore[start] := heuristic_cost_estimate(start, goal)

    while openSet is not empty
        current := the node in openSet having the lowest fScore[] value
        if current = goal
            return reconstruct_path(cameFrom, current)

        openSet.Remove(current)
        closedSet.Add(current)

        for each neighbor of current
            if neighbor in closedSet
                continue

            if neighbor not in openSet
                openSet.Add(neighbor)

            tentative_gScore := gScore[current] + dist_between(current, neighbor)
            if tentative_gScore >= gScore[neighbor]
                continue

            cameFrom[neighbor] := current
            gScore[neighbor] := tentative_gScore
            fScore[neighbor] := gScore[neighbor] + heuristic_cost_estimate(neighbor, goal)
```

> Remark: the above pseudocode assumes that the heuristic function is monotonic
> (or consistent, see below), which is a frequent case in many practical
> problems, such as the Shortest Distance Path in road networks.

Alright so that's us. So what's our function signature look like? We're gonna
get a Path back when we find the path. Except... there might not be a Path. The
pseudocode seems to unwisely ignore that possibility, not even a comment about
it at the end. Ah, well, what can ya do.

Here's our outline:

```rust
pub fn a_star(start: Location, goal: Location) -> Option<Path> {
  None
}
```

Okay, now we'll setup all our maps and sets

```rust
pub fn a_star(start: Location, goal: Location) -> Option<Path> {
  let mut closed_set = HashSet::new();
  let mut open_set = HashSet::new();
  open_set.insert(start);
  let mut came_from = HashMap::new();
  let mut g_score = HashMap::new();
  g_score.insert(start, 0);
  let heuristic_cost_estimate = ???;
  let mut f_score = HashMap::new();
  f_score.insert(start, heuristic_cost_estimate(start, goal));
  while !open_set.is_empty() {
    unimplemented!()
  }
  None
}
```

So what's our `heuristic_cost_estimate`? Well, let's remember what it said:

> The heuristic is problem-specific. For the algorithm to find the actual
> shortest path, the heuristic function must be admissible, meaning that it
> never overestimates the actual cost to get to the nearest goal node.

So it seems like we _could_ say that the cost estimate is always 1. If we wanted
to, it would at least work and give a correct path. The downside is that then we
wouldn't sort our next guesses very well, so we'd search a lot of dead ends
before we got there. So we want a better guess than that. Right now you can only
step orthogonally (a fancy word for "non-diagonal, like a rook moves"), but I
don't want to go back and fiddle this code in too many places if we decide to
change that. So, we'll just pick the minimum x delta and y delta. That'll
usually be an underestimate, unless you're 8-way aligned with your goal (like a
queen moves), in which case it'll still be exactly correct without doing an
over-estimate. And it's correct regardless of if we allow diagonal movement in
the game.

```rust
  let heuristic_cost_estimate = |a: Location, b: Location| (a.x - b.x).abs().min((a.y - b.y).abs());
```

Let's fill in a bit of a `while` loop.

```rust
  while !open_set.is_empty() {
    let current = unimplemented!("how do we do this one?");
    if current == goal {
      return Some(reconstruct_path(came_from, current));
    } else {
      open_set.remove(&current);
      closed_set.insert(current);
      for neighbor in current.neighbors() {
        unimplemented!()
      }
    }
  }
```

Okay, so we've got two problems. First, we need to find "the node in openSet
having the lowest fScore[] value", and next we need to add a `neighbors` method
to the `Location` type. The second one seems simpler to do.

```rust
// part of the `impl Location` block in `lib.rs` {
  pub fn neighbors(self) -> ?? {
    unimplemented!()
  }
```

So, the `a_star` function, as it happens to be written at the moment, expects to
use `current.neighbors()` in a `for` loop. A Rust `for` loop is basically sugar
over use of the
[IntoIterator](https://doc.rust-lang.org/std/iter/trait.IntoIterator.html) trait
with a `while` loop. So we want to throw out something that implements that
trait. If only we could just say that the return value [implements some
trait](https://github.com/rust-lang/rust/issues/34511) and then let Rust figure
out what we mean. Oh, wait, _we can_.

```rust
  pub fn neighbors(self) -> impl IntoIterator<Item = Location> {
    [
      Location { x: self.x + 1, y: self.y },
      Location { x: self.x - 1, y: self.y },
      Location { x: self.x, y: self.y - 1 },
      Location { x: self.x, y: self.y + 1 },
    ]
  }
```

Oh no, the IDE is showing a lot of red lines...

```
D:\dev\roguelike-tutorial-2018>cargo build
   Compiling roguelike-tutorial-2018 v0.4.0-pre (file:///D:/dev/roguelike-tutorial-2018)
error[E0277]: the trait bound `[Location; 4]: std::iter::Iterator` is not satisfied
  --> src\lib.rs:30:29
   |
30 |   pub fn neighbors(self) -> impl IntoIterator<Item = Location> {
   |                             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ `[Location; 4]` is not an iterator; maybe try calling `.iter()` or a similar method
   |
   = help: the trait `std::iter::Iterator` is not implemented for `[Location; 4]`
   = note: required because of the requirements on the impl of `std::iter::IntoIterator` for `[Location; 4]`
   = note: the return type of a function must have a statically known size

error: aborting due to previous error

For more information about this error, try `rustc --explain E0277`.
error: Could not compile `roguelike-tutorial-2018`.
```

Oh, okay, so we just call `.iter()` on the array?

```
error[E0271]: type mismatch resolving `<std::slice::Iter<'_, Location> as std::iter::IntoIterator>::Item == Location`
  --> src\lib.rs:30:29
   |
30 |   pub fn neighbors(self) -> impl IntoIterator<Item = Location> {
   |                             ^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^^ expected reference, found struct `Location`
   |
   = note: expected type `&Location`
              found type `Location`
   = note: the return type of a function must have a statically known size
```

Oh, so we just change the `Item` type to be `&Location` instead.

```
error[E0106]: missing lifetime specifier
  --> src\lib.rs:30:54
   |
30 |   pub fn neighbors(self) -> impl IntoIterator<Item = &Location> {
   |                                                      ^ expected lifetime parameter
   |
   = help: this function's return type contains a borrowed value, but there is no value for it to be borrowed from
   = help: consider giving it a 'static lifetime
```

Augh. Okay, we'll... make the method accept `&self` instead of `self`?

```
error[E0597]: borrowed value does not live long enough
  --> src\lib.rs:31:5
   |
31 | /     [
32 | |       Location { x: self.x + 1, y: self.y },
33 | |       Location { x: self.x - 1, y: self.y },
34 | |       Location { x: self.x, y: self.y - 1 },
35 | |       Location { x: self.x, y: self.y + 1 },
36 | |     ].iter()
   | |_____^ temporary value does not live long enough
37 |     }
   |     - temporary value only lives until here
```

Oh no. Okay time to slow down and think. So the basic problem is that we're
trying to move data across stack frames without giving it any place to live.
Rust demands to know where we think all the data is going to live. So, we'll
make a little struct for that.

```rust
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
```

Alright now we can write what we basically wanted to write for
`Location::neighbors` from the start

```rust
  pub fn neighbors(&self) -> impl Iterator<Item = Location> {
    LocationNeighborsIter {
      x: self.x,
      y: self.y,
      index: 0,
    }
  }
```

(Note that an `Iterator` automatically implements `IntoIterator` by just
returning itself when you call `.into_iter()`.)

So now the `a_star` function works, and if we decide to adjust how you can move
later in the game, we just adjust what nodes are considered the `neighbors` in
one place, and a_star will use the new definition properly. Doing well so far.

Except for the part where we find that minimum `f_cost` thing. Iterators have
done us some magic before, can we do more magic? Heckkin yeah we can. Finding
the minimum is a kind of
[fold](https://en.wikipedia.org/wiki/Fold_(higher-order_function)) "(also termed
reduce, accumulate, aggregate, compress, or inject)". We're taking the whole set
and "smushing" it down into a single value. Iterators have a
[fold](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.fold)
method to do that nicely.

```
fn fold<B, F>(self, init: B, f: F) -> B 
where
    F: FnMut(B, Self::Item) -> B, 
```

Oof, well, maybe not so nicely? No use complaining much, we choose to build
roguelikes, and do the other things, not because they are easy, but because they
are hard!

Let's fill it in a little bit and see what it's like.

```rust
    let current = open_set.iter().fold(panic!("init_type"), |init_type, iter_type| unimplemented!());
```

So, our init type is... `Option<Location>`, I guess? We don't really have an
initial value, so we'll just do that to start. Then for each pass we've got two
cases. Either there isn't an old location, and we automatically accept the new
location, or there is an old location and we compare it with the new location
and pick the lower one.

```rust
    let current = open_set.iter().fold(None, |opt_location, new_loc_ref| match opt_location {
      None => Some(*new_loc_ref),
      Some(old_loc) => if f_score[&old_loc] < f_score[new_loc_ref] {
        Some(old_loc)
      } else {
        Some(*new_loc_ref)
      },
    });
```

"Lokathor", I hear you grumble, "That's stupid! Surely finding the minimim value
in an iterator with some custom key is a common enough operation that it'd be in
the standard library without needing to write a custom fold every time!"

Of course! We just wrote out the `fold` to see how you'd do it yourself if you
had to. But as you guessed there is a
[min_by_key](https://doc.rust-lang.org/std/iter/trait.Iterator.html#method.min_by_key)
method already, and in our actual code we'll use that because it makes the
intent much clearer.

```rust
    let current = *open_set
      .iter()
      .min_by_key(|loc_ref| f_score[loc_ref])
      .expect("the open set should not have been empty because of the loop condition.");
```

Now we can fill in that inner `for` loop. The very first thing we do is continue
if the neighbor is in the `closed_set`, but we can just add a filter onto our
loop's iterator.

```rust
      for neighbor in current.neighbors().filter(|loc_ref| !closed_set.contains(loc_ref)) {
        unimplemented!()
      }
```

Next we add the neighbor into the open set if it's not in the open set. Except
that `HashSet::insert` automatically handles duplicates for us, so we'll add the
neighbor unconditionally.

```rust
      for neighbor in current.neighbors().filter(|loc_ref| !closed_set.contains(loc_ref)) {
        open_set.insert(neighbor);
        unimplemented!()
      }
```

Now we actually process the neighbor by seeing if it's a potential better path.
This involves using the g_score of where _we are_, plus the distance to step
where _where we want to go_, and then checking against the g_score _of this
neighbor_. Should be simple.

```rust
      for neighbor in current.neighbors().filter(|loc_ref| !closed_set.contains(loc_ref)) {
        open_set.insert(neighbor);
        let tentative_g_score = g_score[&current] + dist_between(current, neighbor);
        if tentative_g_score > g_score[&neighbor] {
          continue;
        } else {
          came_from.insert(neighbor, current);
          g_score.insert(neighbor, tentative_g_score);
          f_score.insert(neighbor, g_score[&neighbor] + heuristic_cost_estimate(neighbor, goal));
        }
      }
```

Ah! but we've forgotten an important step! Our g_score and f_score values are
supposed to default to infinity! We'll already have a g_score for current
(because we only got to the current node from having added it as a neighbor in a
past loop, or at the start, which also gives a g_score value), but when we look
up the g_score for the neighbor for the first time we might not have put
anything there yet.

```rust
      for neighbor in current.neighbors().filter(|loc_ref| !closed_set.contains(loc_ref)) {
        open_set.insert(neighbor);
        let tentative_g_score = g_score[&current] + dist_between(current, neighbor);
        if tentative_g_score >= *g_score.entry(neighbor).or_insert(::std::i32::MAX) {
          continue;
        } else {
          came_from.insert(neighbor, current);
          g_score.insert(neighbor, tentative_g_score);
          f_score.insert(neighbor, g_score[&neighbor] + heuristic_cost_estimate(neighbor, goal));
        }
      }
```

And... what's `dist_between`? Well, right now all movement costs are 1, and
roguelikes don't _usually_ use movement costs, so we'll just use 1 for that.
However, since we're also going to be at the MAX value some of the time, we'll
be sure to do a `saturating_add` to avoid any rollover.

```rust
        let tentative_g_score = g_score[&current].saturating_add(1);
```

Ah, and rust has a saturating_add method for all the number types, so it doesn't
know what type we want for that. The fix is that we need to specify the type of
0 that we first added to our `g_score` map earlier.

```rust
  let mut g_score = HashMap::new();
  g_score.insert(start, 0i32);
```

Okay now well... oh, we're out of `unimplemented!()` uses. I guess we're done.
So... right, we just need to call this code. First we'll make all the monsters
magically know where the hero is and path toward them. Remember that right now
the monster takes a step like this:

```rust
// part of GameWorld::run_world_turn in lib.rs
            let move_target = loc + match self.gen.next_u32() >> 30 {
              0 => Location { x: 0, y: 1 },
              1 => Location { x: 0, y: -1 },
              2 => Location { x: 1, y: 0 },
              3 => Location { x: -1, y: 0 },
              impossible => unreachable!("u32 >> 30: {}", impossible),
            };
```

So it seems easy to change how we pick the `move_target` value. We'll just make
a path, throw in a double check that the 0th index is our current spot (which is
_I think_ how the paths we get pack work), and then pick the 1st index to move
to.

```rust
            let path = a_star(loc, self.player_location).expect("couldn't find a path");
            debug_assert_eq!(loc, path[0]);
            let move_target = path[1];
```

Okay, so, now every single turn every player in the game will move towards the
player as best as it can, from an unlimited distance away. This will be... not
fast. A* is not cheap as your search space goes up, and we've got a sizable map.
The final game won't have enemies pathing from unlimited distance though, and if
we needed to share pathing work that's what the [Djikstra
maps](http://www.roguebasin.com/index.php?title=The_Incredible_Power_of_Dijkstra_Maps)
that I mentioned let you easily do.

Let's turn it on and move a square...

```
D:\dev\roguelike-tutorial-2018>cargo run
   Compiling roguelike-tutorial-2018 v0.4.0-pre (file:///D:/dev/roguelike-tutorial-2018)
    Finished dev [unoptimized + debuginfo] target(s) in 2.33s
     Running `target\debug\kasidin.exe`
thread 'main' panicked at 'assertion failed: `(left == right)`
  left: `Location { x: 8, y: 55 }`,
 right: `Location { x: 31, y: 5 }`', src\lib.rs:347:13
note: Run with `RUST_BACKTRACE=1` for a backtrace.
error: process didn't exit successfully: `target\debug\kasidin.exe` (exit code: 101)
```

oops! Hmm. Those aren't even close to each other. Time for println debugging!

```rust
            println!("I am at {:?}\nI wanted {:?}\nMy path was: {:?}", loc, self.player_location, path);
            debug_assert_eq!(loc, path[0]);
```

And we give it a `cargo run`...

```
D:\dev\roguelike-tutorial-2018>cargo run
   Compiling roguelike-tutorial-2018 v0.4.0-pre (file:///D:/dev/roguelike-tutorial-2018)
    Finished dev [unoptimized + debuginfo] target(s) in 2.42s
     Running `target\debug\kasidin.exe`
I am at Location { x: 42, y: 14 }
I wanted Location { x: 41, y: 65 }
My path was: [Location { x: 41, y: 65 }, Location { x: 41, y: 64 }, Location { x: 41, y: 63 }, Location { x: 41, y: 62 }, Location { x: 41, y: 61 }, Location { x: 41, y: 60 }, Location { x: 41, y: 59 }, Location {
x: 41, y: 58 }, Location { x: 41, y: 57 }, Location { x: 41, y: 56 }, Location { x: 41, y: 55 }, Location { x: 41, y: 54 }, Location { x: 41, y: 53 }, Location { x: 41, y: 52 }, Location { x: 41, y: 51 }, Location
{ x: 41, y: 50 }, Location { x: 41, y: 49 }, Location { x: 41, y: 48 }, Location { x: 41, y: 47 }, Location { x: 41, y: 46 }, Location { x: 41, y: 45 }, Location { x: 41, y: 44 }, Location { x: 41, y: 43 }, Location { x: 41, y: 42 }, Location { x: 41, y: 41 }, Location { x: 41, y: 40 }, Location { x: 41, y: 39 }, Location { x: 41, y: 38 }, Location { x: 41, y: 37 }, Location { x: 41, y: 36 }, Location { x: 41, y: 35 }, Location { x: 41, y: 34 }, Location { x: 41, y: 33 }, Location { x: 41, y: 32 }, Location { x: 41, y: 31 }, Location { x: 41, y: 30 }, Location { x: 41, y: 29 }, Location { x: 41, y: 28 }, Location { x: 41, y: 27 }, Location { x: 41, y: 26 }, Location { x: 41, y: 25 }, Location { x: 41, y: 24 }, Location { x: 41, y: 23 }, Location { x: 41, y: 22 }, Location { x: 41, y: 21 }, Location { x: 41, y: 20 }, Location { x: 41, y: 19 }, Location { x: 41, y: 18 }, Location { x: 41, y: 17 }, Location { x: 41, y: 16 }, Location { x: 41, y: 15 }, Location { x: 41, y: 14 }, Location { x: 42, y: 14 }]
thread 'main' panicked at 'assertion failed: `(left == right)`
  left: `Location { x: 42, y: 14 }`,
 right: `Location { x: 41, y: 65 }`', src\lib.rs:348:13
note: Run with `RUST_BACKTRACE=1` for a backtrace.
error: process didn't exit successfully: `target\debug\kasidin.exe` (exit code: 101)
```

Ah-ha. Our paths come out _backwards_ from what we expected. It's the
destination at the 0th index and then proceeding back to the origin. That's not
a big deal, we just adjust our expectations a little. Actually, since paths are
two-way and all that, we can even just pass the args into `a_star` in reverse.
We better write down that `a_star` gives reverse order paths though.

Ah, and you know what we forgot? Our `a_star` doesn't filter out locations that
aren't walkable! That's pretty foolish. So we need a new function signature:

```rust
/// Gives the **Reverse Order** path from `start` to `end`, if any.
pub fn a_star<W>(start: Location, goal: Location, walkable: W) -> Option<Path>
where
  W: Fn(Location) -> bool,
{
```

And then when we filter out the neighbors from our for loop we just add an extra
condition to that.

```rust
      for neighbor in current.neighbors().filter(|loc_ref| walkable(*loc_ref) && !closed_set.contains(loc_ref)) {
```

Alright now we just update how we call `a_star`.

```rust
            let path = a_star(self.player_location, loc, |loc| {
              *self.terrain.entry(loc).or_insert(Terrain::Wall) == Terrain::Wall
            }).expect("couldn't find a path");
```

red lines?

```
error[E0500]: closure requires unique access to `self` but `self.creature_list` is already borrowed
   --> src\lib.rs:346:58
    |
332 |     for creature_mut in self.creature_list.iter_mut() {
    |                         ------------------          - borrow ends here
    |                         |
    |                         borrow occurs here
...
346 |             let path = a_star(self.player_location, loc, |loc| {
    |                                                          ^^^^^ closure construction occurs here
347 |               *self.terrain.entry(loc).or_insert(Terrain::Wall) == Terrain::Wall
    |                ---- borrow occurs due to use of `self` in closure
```

Oh flip off Rust. Stop being a moron. It's clearly safe to edit the terrain when
we're also editing the creature list! But, fine, _we'll baby you because you're
stupid_.

```rust
            let path = {
              let terrain_mut_ref = &mut self.terrain;
              a_star(self.player_location, loc, |loc| {
                *terrain_mut_ref.entry(loc).or_insert(Terrain::Wall) == Terrain::Wall
              }).expect("couldn't find a path")
            };
```

More red lines?

```
error[E0387]: cannot borrow data mutably in a captured outer variable in an `Fn` closure
   --> src\lib.rs:349:18
    |
349 |                 *terrain_mut_ref.entry(loc).or_insert(Terrain::Wall) == Terrain::Wall
    |                  ^^^^^^^^^^^^^^
    |
help: consider changing this closure to take self by mutable reference
```

Okay, that's _kinda_ on me. We only accept `Fn(Location) -> bool` because
checking for if a location is walkable _shouldn't be updating anything_. If it
_is_ updating something, that means that... the walkable status of stuff is
random, or the terrain is changing around you as you compute your path... or
something. Now, some fool might go stuff their map into an
[UnsafeCell](https://doc.rust-lang.org/std/cell/struct.UnsafeCell.html) if
they're really determined, we can't save you from yourself if you're determined.
We can attempt to impose some sanity though, so we _won't_ change `a_star`. In
this case, we'll declare the error to be on the caller of `a_star`. So they're
the ones who will have to do the extra little dance.

```rust
            let path = {
              let terrain_ref = &self.terrain;
              a_star(self.player_location, loc, |loc| {
                terrain_ref.get(&loc).unwrap_or(&Terrain::Wall) == &Terrain::Wall
              }).expect("couldn't find a path")
            };
```

So... we're ready to go I think?

```
D:\dev\roguelike-tutorial-2018>cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.06s
     Running `target\debug\kasidin.exe`
thread 'main' panicked at 'couldn't find a path', libcore\option.rs:960:5
note: Run with `RUST_BACKTRACE=1` for a backtrace.
error: process didn't exit successfully: `target\debug\kasidin.exe` (exit code: 101)
```

Oh, rip. We're saying that things are walkable if they `== &Terrain::Wall`, but
we actually want the opposite. Better fix that and try again.

Oh goddess save us I didn't remove the debug printy line! delete that.

Wow that's still crazy slow. If we add a "turn done!" line at the end of
`move_player` it's like 10 seconds to do a single turn. Let's try it with `cargo
run --release`? Okay, that's not good but it's _responsive_ at least. Eventually
Kasidin gets trapped by all the little `k`s that swarm in.

So let's restrict them to only seeing a little bit of space around them. We'll
say that they can see up to 7 squares only. Just picking a small number that
sounds good (FOV is also expensive, everything is expensive). If the player
isn't visible we'll go back to taking a random step.

```rust
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
```

Wow, our FOV _really should_ have used `Location` throughout. Eh. Not really our
goal to fix that right now.

Give that a run and we're back to having a functional game even in debug mode.
Now we can actually, you know, add fighting.

### Part 06b: Mortal Kombat (with a 'k')

TODO combat

## Part 07: Creating The Interface

TODO display some info i guess
