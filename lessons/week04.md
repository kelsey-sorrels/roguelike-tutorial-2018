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

![image](https://upload.wikimedia.org/wikipedia/commons/thumb/0/0c/SRI_Shakey_with_callouts.jpg/377px-SRI_Shakey_with_callouts.jpg)

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

"Lokathor", I hear you grumble, "That's stupid! Surely finding the minimum value
in an iterator with some custom key is a common enough operation that it'd be in
the standard library without needing to write a custom fold every time!"

Of course! We just wrote out the `fold` to see how you'd do it yourself if you
had to. But, as you guessed, there is a
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

Okay now we... oh, we're out of `unimplemented!()` uses. I guess we're done.
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

Okay, that one is _kinda_ on me. We only accept `Fn(Location) -> bool` because
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

Oh, rip. We're saying that things are walkable if they are `== &Terrain::Wall`,
but we actually want the opposite. Better fix that and try again.

Oh goddess save us I didn't remove the debug printy line! delete that and try
again again.

Wow that's still crazy slow. If we add a "turn done!" line at the end of
`move_player` it's like 10 seconds or so to do a single turn. Let's try it with
`cargo run --release`? Okay, that's not perfect but it's _responsive_ at least.
Eventually Kasidin gets trapped by all the little `k`s that swarm in. And you
can't fight them yet, so you're just stuck.

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

Give that a run and we're back to having a responsive game even in debug mode.
Now we can actually, you know, add fighting.

[code so far](https://github.com/Lokathor/roguelike-tutorial-2018/commit/243906ec243bf2492cbbec0ebbf6844ba8d4cb5c)

### Part 06b: Mortal Kombat (with a 'k')

Time to add some combat!

**So, what's our combat system?** Some sort of vaguely d20 thing? Roll 1d20+Atk
>= Target.AC to hit, and then 1d6 for damage off of their HP? We've seen it a
billion times. Let's try something new! We'll rip off a _different_ old RPG's
mechanics instead of stealing from DnD.

#### Part 06b.a: Module Cleanup

So since we're going to be doing more random stuff we'll make a module just for
that. We move all the old prng code into `prng.rs`. We've also gotta make a
`new` method for `PCG32`, since `state` is module private. It doesn't _do_
anything, just passes on the param.

```rust
// part of impl PCG32 in prng.rs
  pub fn new(state: u64) -> Self {
    Self { state }
  }
```

Next thing we want is to make some const rand ranges for the normal sorts of
dice we'll want to roll.

Except we can't call `RandRangeInclusive32::new` in a const context! Easy, we
just make a test that the const value is the same as the output from
`RandRangeInclusive32::new` (since test _don't_ run in a const context of
course), and we fill in the initial const value with 0s. We let the test fail,
and it'll print out the values that it was looking for.

```rust
pub const d4: RandRangeInclusive32 = RandRangeInclusive32 {
  base: 0,
  width: 0,
  reject: 0,
};
#[test]
fn test_d4_const_is_correct() {
  assert_eq!(d4, RandRangeInclusive32::new(1..=4))
}
```

and give that a `cargo test`

```
running 4 tests
test prng::range_range_inclusive_32_sample_validity_test_d6 ... ignored
test precise_permissive_fov::view_tests ... ok
test prng::test_d4_const_is_correct ... FAILED
test precise_permissive_fov::line_tests ... ok

failures:

---- prng::test_d4_const_is_correct stdout ----
thread 'prng::test_d4_const_is_correct' panicked at 'assertion failed: `(left == right)`
  left: `RandRangeInclusive32 { base: 0, width: 0, reject: 0 }`,
 right: `RandRangeInclusive32 { base: 1, width: 4, reject: 4294967291 }`', src\prng.rs:114:3
note: Run with `RUST_BACKTRACE=1` for a backtrace.


failures:
    prng::test_d4_const_is_correct

test result: FAILED. 2 passed; 1 failed; 1 ignored; 0 measured; 0 filtered out

error: test failed, to rerun pass '--lib'
```

Bam! Now we'll throw in some for the other dice.

```
---- prng::test_d12_const_is_correct stdout ----
thread 'prng::test_d12_const_is_correct' panicked at 'assertion failed: `(left == right)`
  left: `RandRangeInclusive32 { base: 1, width: 4, reject: 4294967291 }`,
 right: `RandRangeInclusive32 { base: 1, width: 12, reject: 4294967291 }`', src\prng.rs:156:3

---- prng::test_d10_const_is_correct stdout ----
thread 'prng::test_d10_const_is_correct' panicked at 'assertion failed: `(left == right)`
  left: `RandRangeInclusive32 { base: 1, width: 4, reject: 4294967291 }`,
 right: `RandRangeInclusive32 { base: 1, width: 10, reject: 4294967289 }`', src\prng.rs:146:3
note: Run with `RUST_BACKTRACE=1` for a backtrace.

---- prng::test_d20_const_is_correct stdout ----
thread 'prng::test_d20_const_is_correct' panicked at 'assertion failed: `(left == right)`
  left: `RandRangeInclusive32 { base: 1, width: 4, reject: 4294967291 }`,
 right: `RandRangeInclusive32 { base: 1, width: 20, reject: 4294967279 }`', src\prng.rs:166:3

---- prng::test_d6_const_is_correct stdout ----
thread 'prng::test_d6_const_is_correct' panicked at 'assertion failed: `(left == right)`
  left: `RandRangeInclusive32 { base: 1, width: 4, reject: 4294967291 }`,
 right: `RandRangeInclusive32 { base: 1, width: 6, reject: 4294967291 }`', src\prng.rs:126:3

---- prng::test_d8_const_is_correct stdout ----
thread 'prng::test_d8_const_is_correct' panicked at 'assertion failed: `(left == right)`
  left: `RandRangeInclusive32 { base: 1, width: 4, reject: 4294967291 }`,
 right: `RandRangeInclusive32 { base: 1, width: 8, reject: 4294967287 }`', src\prng.rs:136:3
```

So helpful! And we'll also update
`range_range_inclusive_32_sample_validity_test_d6` to use the actual `d6` value
now that we have one. While we're at it, we should test the other dice too.
Let's start with `d8`...

```
---- prng::range_range_inclusive_32_sample_validity_test_d8 stdout ----
thread 'prng::range_range_inclusive_32_sample_validity_test_d8' panicked at 'assertion failed: outputs[0] < 8', src\prng.rs:128:3
note: Run with `RUST_BACKTRACE=1` for a backtrace.
```

Wait what? Oh, we kinda talked about this before. Because 8 is a power of 2,
it'll actually have a _full_ range rejected on the high end, so there should be
exactly 8 elements in that outputs[0] slot, instead of being less than 8
elements in the rejected slot. I think? We'll just give that a bump into
`assert!(outputs[0] == 8);` and try again.

```
D:\dev\roguelike-tutorial-2018>cargo test --release -- --ignored
   Compiling roguelike-tutorial-2018 v0.4.0-pre (file:///D:/dev/roguelike-tutorial-2018)
    Finished release [optimized] target(s) in 8.37s
     Running target\release\deps\roguelike_tutorial_2018-a493e670302f602a.exe

running 2 tests
test prng::range_range_inclusive_32_sample_validity_test_d8 ... ok
test prng::range_range_inclusive_32_sample_validity_test_d6 ... ok
```

Okay. So now we want this sort of thing for all the dice. We _could_ make a
macro to carefully eliminate the code duplication, but that'd actually be harder
than copy-pasting, so we'll just copy-paste and fiddle things until we've got
one for each dX const.

```
D:\dev\roguelike-tutorial-2018>cargo test --release -- --ignored
   Compiling roguelike-tutorial-2018 v0.4.0-pre (file:///D:/dev/roguelike-tutorial-2018)
    Finished release [optimized] target(s) in 8.37s
     Running target\release\deps\roguelike_tutorial_2018-a493e670302f602a.exe

running 6 tests
test prng::range_range_inclusive_32_sample_validity_test_d8 ... ok
test prng::range_range_inclusive_32_sample_validity_test_d4 ... ok
test prng::range_range_inclusive_32_sample_validity_test_d10 ... ok
test prng::range_range_inclusive_32_sample_validity_test_d6 ... ok
test prng::range_range_inclusive_32_sample_validity_test_d12 ... ok
test prng::range_range_inclusive_32_sample_validity_test_d20 ... ok
```

Alright, so now we're _reasonably_ assured that our `d4` through `d20` random
ranges are doing what we think they're doing.

**Next step:** We'll make a function that rolls a dX, then rolls again and adds
to the total if you get the X value. We'll call this `explode`, since the value
"explodes" into the next higher tier when you get a maximum output. There's
three main places we could put this code: a method on the `RandRangeInclusive32`
type, a method on the `PCG32` type, or just a free function in the `prng.rs`
module. The `RandRangeInclusive32` type already has a `roll_with` method, so we
could put `explode` there.

```rust
// part of impl RandRangeInclusive32
  /// An "explosive" style roll of the dice.
  ///
  /// Does one roll, and if the result is the maximum then instead of directly
  /// returning another roll is performed (which can also explode). The final
  /// output is the last non-maximum roll plus the number of explosions times
  /// the maximum value of a single roll.
  ///
  /// There's no direct limit to the number of re-rolls, but in practice the
  /// reroll count is unlikely to ever be that large.
  ///
  /// This is _intended_ for ranges where the minimum is 1, but actually there's
  /// nothing preventing you from using it with other ranges.
  pub fn explode(&self, gen: &mut PCG32) -> u32 {
    let highest = self.high();
    let mut explosions = 0;
    loop {
      if let Some(output) = self.convert(gen.next_u32()) {
        if output == highest {
          explosions += 1;
        } else {
          return output + explosions * highest;
        }
      }
    }
  }
```

**Next Step:** We make a function that takes an input and rolls a number
according to the other fantasy RPG we'll be taking our mechanics from (remember
that [game mechanics aren't protected by
copyright](https://www.gamasutra.com/view/news/273935/Texas_court_affirms_game_mechanics_not_protected_under_copyright_law.php),
so we're in the clear _legally_, even if we are potentially being lazy
_morally_). We don't need to say what game, we'll just refer to it as "the game"
when we talk about it here. You can go investigate what game if you want, it
shouldn't be hard at all.

Our magical input to output function will be called `step4`. It needs a `PCG32`
to use and a step number to roll. Where does this code live? Well, it's not
using any particular `RandRangeInclusive32` value, and as I said I think the
`PCG32` type should stay as dead simple as possible, so we'll just make it a
free function.

```rust
/// Rolls a step roll, according to the 4th edition chart.
pub fn step4(gen: &mut PCG32, step: ?) -> ? {
  unimplemented!()
}
```

We just need to decide what _type_ we want our input and output to be. Hmm. Well
the step inputs will _by default_ be positive, but if there's a lot of modifiers
on a roll, you might get a total that's less than 0. Similarly, a modifier will
sometimes need to be a negative number.

Let's review the status of wrapping math in rust:

* As I mentioned when we first made the PCG32, rust's math operators (`+`, `-`,
  etc) will, by default, panic on wrap around _in debug mode only_, and then do
  wrapping math in release mode.
* You can also call specific methods if you want always wrapping, always
  saturating, or always checked (which is what we do with `PCG32`).
* You can _also_ also use the
  [Wrapping](https://doc.rust-lang.org/std/num/struct.Wrapping.html) type to
  make the normal operator symbols always do wrapping math. It's clear enough
  that a `Checked` type wrapper wouldn't work (the signatures it uses wouldn't
  work with the the operator traits), but for whatever reason there isn't a
  `Saturating` type wrapper, now that I'm looking for it in the standard
  library. Weird. I guess no one cared enough to write it.
* You can _also also_ also just straight up force math checks to be enabled or
  disabled regardless of your compilation mode by overriding the default of
  "checked math on = debug mode on" to be "checked math on" or "checked math
  off" (eg: `-Z force-overflow-checks=off`). I don't like this last method so
  much because it's not obvious when reading just the code that you're expecting
  some special flag to be in effect.

So, I think that we'll just accept `i32` for input, and also give output `i32`
for output. The _actual_ outputs will never be less than 0, but if the output
type and input type match up it's a lot easier to mix around numbers if you need
to (which is a _good_ thing in this case, even though in other situations it can
obviously be [very bad](http://articles.latimes.com/1999/oct/01/news/mn-17288)).

```rust
/// Rolls a step roll, according to the 4th edition chart.
pub fn step4(gen: &mut PCG32, step: i32) -> i32 {
  unimplemented!()
}
```

Now, the step roller's actual minimum allowed input is 1. If the input is less
than 1, the output will be 0.

```rust
/// Rolls a step roll, according to the 4th edition chart.
pub fn step4(gen: &mut PCG32, step: i32) -> i32 {
  if step < 1 {
    0
  } else {
    unimplemented!()
  }
}
```

For 1 through 13, there's a specific exploding dice expression that will be
returned.

```rust
/// Rolls a step roll, according to the 4th edition chart.
pub fn step4(gen: &mut PCG32, step: i32) -> i32 {
  if step < 1 {
    0
  } else {
    (match step {
      1 => (d4.explode(gen) as i32 - 2).max(1) as u32,
      2 => (d4.explode(gen) as i32 - 1).max(1) as u32,
      3 => d4.explode(gen),
      4 => d6.explode(gen),
      5 => d8.explode(gen),
      6 => d10.explode(gen),
      7 => d12.explode(gen),
      8 => d6.explode(gen) + d6.explode(gen),
      9 => d8.explode(gen) + d6.explode(gen),
      10 => d8.explode(gen) + d8.explode(gen),
      11 => d10.explode(gen) + d8.explode(gen),
      12 => d10.explode(gen) + d10.explode(gen),
      13 => d12.explode(gen) + d10.explode(gen),
      more_than_13 => unimplemented!(),
    }) as i32
  }
}
```

As you can see, mixing in the use of negative numbers is already a little fiddly
with the lowest step values there, so it's probably a good sign to stick with
`i32` as much as we can.

So, if we have a number above 13, we want to add a d12 to the total and then
subtract 7 steps. So step 14 is d12 + step 7 (2d12 total), and step 15 is d12 +
step 8 (d12+2d6 total), and so forth. We could do a loop and then a recursive
call, or we can just do a slightly bigger loop without a recursive call.

Let's try without recursion:

```rust
/// Rolls a step roll, according to the 4th edition chart.
pub fn step4(gen: &mut PCG32, mut step: i32) -> i32 {
  if step < 1 {
    0
  } else {
    let mut total = 0i32;
    loop {
      match step {
        1 => return total + (d4.explode(gen) as i32 - 2).max(1),
        2 => return total + (d4.explode(gen) as i32 - 1).max(1),
        3 => return total + d4.explode(gen) as i32,
        4 => return total + d6.explode(gen) as i32,
        5 => return total + d8.explode(gen) as i32,
        6 => return total + d10.explode(gen) as i32,
        7 => return total + d12.explode(gen) as i32,
        8 => return total + d6.explode(gen) as i32 + d6.explode(gen) as i32,
        9 => return total + d8.explode(gen) as i32 + d6.explode(gen) as i32,
        10 => return total + d8.explode(gen) as i32 + d8.explode(gen) as i32,
        11 => return total + d10.explode(gen) as i32 + d8.explode(gen) as i32,
        12 => return total + d10.explode(gen) as i32 + d10.explode(gen) as i32,
        13 => return total + d12.explode(gen) as i32 + d10.explode(gen) as i32,
        _more_than_13 => {
          total += d12.explode(gen) as i32;
          step -= 7;
        }
      }
    }
  }
}
```

That's... not nice looking :/ If we do the smaller loop... well let's just see...

```rust
/// Rolls a step roll, according to the 4th edition chart.
pub fn step4_recur(gen: &mut PCG32, mut step: i32) -> i32 {
  if step < 1 {
    0
  } else {
    (match step {
      1 => (d4.explode(gen) as i32 - 2).max(1) as u32,
      2 => (d4.explode(gen) as i32 - 1).max(1) as u32,
      3 => d4.explode(gen),
      4 => d6.explode(gen),
      5 => d8.explode(gen),
      6 => d10.explode(gen),
      7 => d12.explode(gen),
      8 => d6.explode(gen) + d6.explode(gen),
      9 => d8.explode(gen) + d6.explode(gen),
      10 => d8.explode(gen) + d8.explode(gen),
      11 => d10.explode(gen) + d8.explode(gen),
      12 => d10.explode(gen) + d10.explode(gen),
      13 => d12.explode(gen) + d10.explode(gen),
      _more_than_13 => {
        let mut total = 0u32;
        while step > 13 {
          total += d12.explode(gen);
          step -= 7;
        }
        total + step4_recur(gen, step) as u32
      }
    }) as i32
  }
}
```

That doesn't feel particularly good either. Well, we can throw the whole module
into [godbolt](godbolt.org) (be sure to replace `use super::*;` with `use
std::ops::*`) and check it. We get... 747 lines of ASM for the recursive
version vs 752 lines of ASM for the non-recursive version (using `-C
opt-level=3`). Uh, maybe throw all that into a diff page, maybe there's... yeah,
no hints there. It's not mostly the same until X point, it's like totally
different.

Well, we can throw it into a benchmarker and pick that way? We just make a file
in the `benches/` directory off of the crate root, we can call it `benches.rs`
as a default.

```rust
#![feature(test)]
#![allow(non_snake_case)]

extern crate test;
use test::Bencher;

extern crate roguelike_tutorial_2018;
use roguelike_tutorial_2018::*;

#[bench]
fn bench_step4(b: &mut Bencher) {
  let gen = &mut PCG32::new(u64_from_time());
  b.iter(|| step4(gen, 20));
}

#[bench]
fn bench_step4_recur(b: &mut Bencher) {
  let gen = &mut PCG32::new(u64_from_time());
  b.iter(|| step4_recur(gen, 20));
}
```

And use `cargo bench`, or `cargo +nightly bench` if your default compiler is
stable.

```
running 2 tests
test bench_step4       ... bench:          13 ns/iter (+/- 0)
test bench_step4_recur ... bench:          13 ns/iter (+/- 0)

test result: ok. 0 passed; 0 failed; 0 ignored; 2 measured; 0 filtered out
```

Welp! Even running that a few more times, they pretty much run the same speed.
Guess we'll use the non-recursive version just because it's clearer. We can
clean it up a bit too probably. Let's try this

```rust
/// Rolls a step roll, according to the 4th edition chart.
pub fn step4(gen: &mut PCG32, mut step: i32) -> i32 {
  if step < 1 {
    0
  } else {
    let mut total = 0;
    while step > 13 {
      total += d12.explode(gen);
      step -= 7;
    }
    (total + match step {
      1 => (d4.explode(gen) as i32 - 2).max(1) as u32,
      2 => (d4.explode(gen) as i32 - 1).max(1) as u32,
      3 => d4.explode(gen),
      4 => d6.explode(gen),
      5 => d8.explode(gen),
      6 => d10.explode(gen),
      7 => d12.explode(gen),
      8 => d6.explode(gen) + d6.explode(gen),
      9 => d8.explode(gen) + d6.explode(gen),
      10 => d8.explode(gen) + d8.explode(gen),
      11 => d10.explode(gen) + d8.explode(gen),
      12 => d10.explode(gen) + d10.explode(gen),
      13 => d12.explode(gen) + d10.explode(gen),
      _other => unreachable!(),
    }) as i32
  }
}
```

Oh, and we'll run the benchmark again real fast just to make sure we didn't
accidentally hurt ourselves somehow.

```
running 1 test
test bench_step4 ... bench:          11 ns/iter (+/- 0)
```

What. the. crap.

_Okay_, so we're using this version I guess.

This concludes the module cleanup, we're ready to roll our dice.

#### Part 06b.b: Looking To Protect Yourself, Or Deal Some Damage?

So what stats does a `Creature` have again?

```rust
pub struct Creature {
  pub icon: u8,
  pub color: u32,
  pub is_the_player: bool,
  pub id: CreatureID,
}
```

Hmm, so we'll give a creature a `hit_points` and `damage_step`, both of which
are `i32`. They default to 10 and 5. If you bump another creature you roll your
damage step and deal that many damage. No attack rolls or anything fancy, we'll
do that in a moment.

First, we want the player to be able to bump an enemy. Where's our player combat
code... oh right:

```rust
    if self.creature_locations.contains_key(&player_move_target) {
      println!("Player does a bump!");
    } else {
```

Alright, so now we care about what that key is if it is in there. We just switch that `if` into a `match` and...

```rust
    match self.creature_locations.get(&player_move_target) {
      Some(creature_id_ref) => {
        println!("Player does a bump!");
      }
      None => {
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
    }
```

We get some red lines!

```
error[E0502]: cannot borrow `self.creature_locations` as mutable because it is also borrowed as immutable
   --> src\lib.rs:331:29
    |
320 |       match self.creature_locations.get(&player_move_target) {
    |             ----------------------- immutable borrow occurs here
...
331 |               let player_id = self
    |  _____________________________^
332 | |               .creature_locations
    | |_________________________________^ mutable borrow occurs here
...
341 |       }
    |       - immutable borrow ends here
```

Oh come on! There's no actual reference that must remain valid in the `None`
case, this should be legit. _Fine_, we'll go back to the entry API, which is how
you're supposed to get around this crap.

Except... we can't do that, because while it helps in the `Occupied` case if we
find a monster, it makes it near impossible to write the `Vacant` case, since
we'll have already grabbed the locations as mutable and we'll need to mutate the
locations to get the player out of their old spot to put them in the new spot.

Maybe the magical [Non-Lexical
Lifetimes](https://github.com/rust-lang/rust-roadmap/issues/16) will save us? We
put this at the top of our file and...

```rust
#![feature(nll)]
```

Welp, we're back to being nightly only I guess, `*edits readme*`.

So now we know the `CreatureID` of who we want to attack. This is one of those
situations where we have to futz with the player's entry in the creature list
and also the target's entry in the creature list. Since right now there's
nothing where the damage can reflect on to the player or anything, we'll just do
all the player's work, then apply it to the target.

```rust
      Some(target_id_ref) => {
        // someone is there, do the attack!
        let player_damage_roll = {
          let player_id_ref = self.creature_locations.get(&self.player_location).unwrap();
          let player_ref = self.creature_list.iter().find(|creature_ref| &creature_ref.id == player_id_ref).unwrap();
          step4(&mut self.gen, player_ref.damage_step)
        };
        let target_ref_mut = self
          .creature_list
          .iter_mut()
          .find(|creature_mut_ref| &creature_mut_ref.id == target_id_ref)
          .unwrap();
        target_ref_mut.hit_points -= player_damage_roll;
      }
```

Okay, now during `run_world_turn`, in addition to skipping anyone who is the
player, we'll also skip creatures that don't have any hit points, and at the end
of the turn we'll clear any NPCs that are out of hit points.

```rust
    self
      .creature_list
      .retain(|creature_ref| creature_ref.hit_points > 0 || creature_ref.is_the_player);
```

Right now the player can't lose, but they also can't take damage. Let's try this.

```
D:\dev\roguelike-tutorial-2018>cargo run
    Finished dev [unoptimized + debuginfo] target(s) in 0.06s
     Running `target\debug\kasidin.exe`
turn over!
turn over!
CreatureID(23) does a bump!
turn over!
CreatureID(3) does a bump!
CreatureID(6) does a bump!
CreatureID(25) does a bump!
turn over!
CreatureID(6) does a bump!
CreatureID(48) does a bump!
turn over!
CreatureID(6) does a bump!
turn over!
CreatureID(14) does a bump!
turn over!
thread 'main' panicked at 'Our locations and list are out of sync!', libcore\option.rs:960:5
note: Run with `RUST_BACKTRACE=1` for a backtrace.
error: process didn't exit successfully: `target\debug\kasidin.exe` (exit code: 101)
```

What went wrong? I bet you can tell me.

We didn't update our creature locations mapping!

So, we need a much smarter call to `retain`.

```rust
    let creature_locations_mut = &mut self.creature_locations;
    self.creature_list.retain(|creature_ref| {
      let keep = creature_ref.hit_points > 0 || creature_ref.is_the_player;
      if !keep {
        let dead_location = *creature_locations_mut
          .iter()
          .find(|&(_, v_cid)| v_cid == &creature_ref.id)
          .expect("Locations list out of sync!")
          .0;
        creature_locations_mut.remove(&dead_location);
      };
      keep
    });
```

Now we can kill stuff and it _doesn't_ crash! Such a win.

#### Part 06b.c: The monsters strike back

Now the monsters should also deal damage. We just gotta update the `// go there`
part of the monster movement code.

```rust
            // go there
            match self.creature_locations.get(&move_target) {
              Some(target_id_ref) => {
                // someone is there, do the attack!
                let our_damage_roll = step4(&mut self.gen, creature_mut.damage_step);
                let target_ref_mut = self
                  .creature_list
                  .iter_mut()
                  .find(|creature_mut_ref| &creature_mut_ref.id == target_id_ref)
                  .unwrap();
                if target_ref_mut.is_the_player {
                  target_ref_mut.hit_points -= our_damage_roll;
                }
                // TODO: log that we did damage.
              }
              None => match *self.terrain.entry(move_target).or_insert(Terrain::Floor) {
                Terrain::Wall => {
                  continue;
                }
                Terrain::Floor => {
                  let id = self.creature_locations.remove(&loc).expect("The creature wasn't where they should be!");
                  let old_id = self.creature_locations.insert(move_target, id);
                  debug_assert!(old_id.is_none());
                }
              },
            }
```

Oh no, red lines. We're editing the creature list while we're iterating the
creature list. Totally no good.

Hmmmmmmm. How do we want to handle this one? Easiest way seems to be to just
grab all the `CreatureID` values, make an initiative list out of that, and then
go over the initiative list. We can also filter the initiative list as we build
it.

```rust
    let initiative_list: Vec<CreatureID> = self
      .creature_list
      .iter()
      .filter_map(|creature_mut| {
        if creature_mut.is_the_player || creature_mut.hit_points < 1 {
          None
        } else {
          Some(CreatureID(creature_mut.id.0))
        }
      })
      .collect();
    for creature_id_ref in initiative_list.iter() {
      //...
    }
```

Now we just adjust all the stuff inside to work with an ID instead of a creature
itself, and then we're off to the races. Let's add some print statements too.

```
turn over!
CreatureID(19) did 6 damage to CreatureID(1)
turn over!
Player did 5 damage to CreatureID(19)
CreatureID(19) did 2 damage to CreatureID(1)
turn over!
Player did 4 damage to CreatureID(19)
CreatureID(19) did 5 damage to CreatureID(1)
```

Neat!

In terms of mechanics, well you'd probably want to make an attack roll and
stuff, but since it's now clear where and how we'd put that in the code (roll
the attack while you roll the damage, then check if the attack is high enough
before you apply the damage), I'll leave that up to you for now.

## Part 07: Creating The Interface

Alright so the player is taking all this damage but they don't know it. That's
our next goal.

It's not a big goal, it's actually super easy if we use _just a bit_ of unsafe code.

First let's go to where we draw the game, and now we draw to only a sub-portion
of the full screen.

```rust
      // draw the map, save space for the status line.
      const STATUS_HEIGHT: usize = 1;
      let full_extent = (ids.width(), ids.height());
      let map_view_end = (full_extent.0, full_extent.1 - STATUS_HEIGHT);
      for (scr_x, scr_y, id_mut) in ids.slice_mut((0, 0)..map_view_end).iter_mut() {
```

Next we can write text to the status line at the top by turning a single row of
the `ImageMutSlice<u8>` image into a `&mut [u8]` and then writing into it with
plain string formatting. Rust can't verify when you cast between pointer types
like this, so we'll want to be extra careful that we've done our math right with
some extra debug asserts.

```rust
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
      write!(status_line_u8_slice_mut, "HP: {}", player_hp).ok();
```

And now we get a lovely game with a status bar and everything:

![game-working](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week04-01.png)

We can even add a little enemy counter, so that you know when you've killed
everything.

```rust
      write!(status_line_u8_slice_mut, "HP: {}, Enemies: {}", player_hp, game.creature_list.len() - 1).ok();
```

![game-working](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week04-02.png)

Anyway, that's it for this week I think. I did all of this in one big marathon,
OBS says that I was live for about 7h:40m. It'll probably take you a lot less
time than that to read it I hope.
