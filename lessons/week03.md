# Week 03

## Part 04: Field of View

So this week we have to compute some Field of View (FOV). The idea is simple:
Because we can't see through walls, and because there are walls, we can only see
part of the map. We will use the Precise Permissive FOV algorithm, because the
only published [FOV
comparison](http://www.roguebasin.com/index.php?title=Comparative_study_of_field_of_view_algorithms_for_2D_grid_based_worlds)
rates PPFOV as being the best FOV that's fully symmetric. I'm a fan of having
proper tactical battles, so I think that non-symmetric FOV is a farce, and also
a crime against your players.

So, the [overview
page](http://www.roguebasin.com/index.php?title=Permissive_Field_of_View) weakly
outlines some advantages and disadvantages, it also links to implementations in
various languages. The [algorithm
page](http://www.roguebasin.com/index.php?title=Precise_Permissive_Field_of_View)
explains how we actually do it. There's one big catch: there's supposed to be
some images on that article, but the wiki itself [doesn't allow image
uploads](http://www.roguebasin.com/index.php?title=Talk:Precise_Permissive_Field_of_View)
and they keep eventually going down from wherever they're hosted elsewhere each
time they're hosted elsewhere. Some of the diagrams are ASCII art within the
article, and some of the images I happen to just remember how they look and I'll
draw them out, but for the rest we'll just have to fumble in the dark. There's a
[python
version](http://www.roguebasin.com/index.php?title=Permissive_Field_of_View_in_Python)
of the code that we can also look at as we fumble through. Python is not a good
programming language, and you should strive to avoid it when you can, but the
particular example there makes for good pseudo-code.

### Part 04a: What Will We Build

So, Field of View allows you to do some math to determine what positions
can be seen from what other positions. There's many types of FOV that can be
used. Precise Permissive Field of View (PPFOV) has the following key properties:

* **2-dimensional:** This is fine for us, since our game exists in distinct cave
  layers. The algorithm could _theoretically_ be expanded out into 3d (as could
  most FOV), but all known examples and the explanation text itself assume 2d.
* **Binary, Grid Based:** Every single map cell is either fully blocking of FOV
  or fully non-blocking of FOV. We don't compute data about partial cover around
  corners, or partial visibility as you look into a fog, or any other such thing.
* **Fully Reflexive:** If you can see square A from square B, you'll always be
  able to see square B from square A as well. This sounds simple, but several of
  the popular FOV algorithms _don't_ have this property (because it lets you
  compute FOV faster). I usually like speed, but I don't like speed at the
  expense of accuracy.
* **Very Permissive:** If _any_ part of square A can see _any_ part of square B,
  then B will be in the result set for A's FOV. This is part of how the "fully
  reflexive" part works, because in some situations a corner or center of A will
  be able to see part of B, but not the other way around. If you only compute
  from corners you'll end up with situations where you can't see things you
  should be able to.
* **You _Can_ See Through Diagonal Gaps:** Our cave generators don't make many
  of them right now, but it can happen. You also can't currently move
  diagonally, so you can't step through a gap that you can see through. We might
  add diagonal movement later.
* **You _Can Not_ See Entirely Around Pillars:** This should be obvious, but
  sometimes it helps to state the seemingly obvious.

Let's look at some diagrams:

> S can always see D, no matter how long you make the intermediate corridor
> here.

```
                   ####
 ###################..d
##...................## 
s..################### 
####
```

> S can see D through the "diagonal gap".

```
#d
s#
```

> S _cannot_ see D because it's on the far side of the pillar.

```
..d
.#.
s..
```

### Part 04b: Scanning One Quadrant

So, to make the process simpler we will create a way to scan over a single
quadrant out from a source square, and then for the other quadrants we'll use
the necessary reflections as we add the origin to the "current location" delta.

We start by assuming that the origin can be seen, and then we scan backwards and
upwards along a [series of -1 slope
lines](http://www.wolframalpha.com/input/?i=y+%3D+-x), like this:

```
9
5 8
2 4 7
@ 1 3 6
```

For all position coordinates, we refer to the _lower left_ corner of the square
as being the position for that square.

![labeled-squares](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week03-01.png)

* A is (0,0), B is (1,0), C is (0,1), and D is (1,1).
* Any time we talk about the top-left corner of (x,y), we're talking about (x,y+1)
* Any time we talk about the bottom-right corner of (x,y), we're talking about (x+1,y)

As we scan, we maintain a list of "Views". A view has two "sight lines":

* One goes from the bottom-right of the origin to some sufficiently high point
  on the Y-axis. This is the "steep" line.
* One goes from the top-left of the origin to some sufficiently far point along
  the X-axis. This is the "shallow" line.
* The relative positioning of the two lines is invariant. The ending of the
  steep line will always be above the shallow line, and the ending of the
  shallow line will always be below the steep line.

![initial-sight-lines](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week03-02.png)

In addition, each View stores a list (per line) of the squares that have blocked
sight and forced a view update during the scanning. These squares are called
"bumps". We need to store the bumps because in some cases a an initial update to
a sight line might make it intersect a previous bump, and then we would need to
update it again.

For each square, we first determine what View it is within. At first there is a
single View, but as you'll see in a moment you can end up with more than one.
Each View will cover a fully disjoint portion of the quadrant being scanned, so
no square will be in more than one view. However, it's also possible that a
square will be in no view at all, which indicates that some other square was "in
front of" the square in question (relative to this FOV's origin), and so we must
skip past it.

Once we've determined what View the square lies within, we also check if the
square blocks vision or not. If the square _does not_ block vision then we mark
it as seen and move on. If the square _does_ block vision, we have to determine
in what way the square is touching our selected view, and then update that view
accordingly.

* If the square _overlaps one_ of the sight lines but not the other, it "bumps"
  the sight line inward, narrowing the visible space. If the steep line is
  overlapped then we bump the steep line down, and if the shallow line is
  overlapped we bump the shallow line up.
* If the square _overlaps both_ of the sight lines, then that view is completely
  blocked by the square, and we remove that View from all further scanning.
* If the square _sits between_ the two sight lines, we split the selected View
  into two Views. To do this, we first clone the View and place the clone at the
  very next index within our list. Then for the lower indexed View we adjust the
  steep line down, and for the higher indexed View we adjust the shallow line
  up. This way, the views are always kept ordered, counter-clockwise, by the
  area that they cover.

This is a lot, let's try an example. We've got some squares that we'll scan,
let's number them in order like before.

![view-scan](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week03-03.png)

And put some sight lines on the grid. Again like before, red is our shallow line, and blue is our steep line.

![view-scan-sight-lines](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week03-04.png)

So we scan several squares that all don't block sight, until we come to number
5, and it does block sight. Since 5 is intersecting just one sight line of the
View, that means that we are in the case where the steep line is "bumped"
inward. The shallow line is unaffected by this.

![five-is-a-wall](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week03-05.png)

Now we continue our scan and eventually find that 7 is also a blocker. It's
within the View we've got going on, so we split clone the view, and the lower
index view treats it as a steep line bump, while the higher index view treats it
as a shallow bump. Our lower index view remains red and blue, but our new higher
index view will be yellow and purple.

![seven-of-wall](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week03-06.png)

Hopefully it makes some sense now.

As I said earlier, the only snag is if we move a line and it end up hitting a
previously scanned bump. To handle this, every time we move a _shallow_ line we
have to also check it against all of the previous _steep_ bumps, and vice versa.
You have to remember that one line has to check against _the other line's bumps_
when going back over old bumps. It gave me a lot of trouble to remember that
detail the first time I tried to implement PPFOV. And also the second time too.
By the third time I remembered at least.

One final note, if a View has both lines colinear with each other, and they're
also projecting out from either extremity of the FOV origin, then that also
counts as a dead view (in addition to the case where both lines intersect the
same square).

We stop a quadrant scan when we run out of Views in our list of active views. We
also stop when we reach the limits of the FOV range even if there are still
active views.

### Part 04c: Implementing... All That Stuff

So, remembering that there's a [python
example](http://www.roguebasin.com/index.php?title=Permissive_Field_of_View_in_Python)
to help guide us, which was itself created based on [the C and C++
version](http://www.roguebasin.com/index.php?title=Permissive-fov), let's start
writing some of this in Rust.

We start by defining a sight line type, and associated methods.

```rust
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
  fn p_below(&self, x: i32, y: i32) -> bool {
    self.relative_slope(x, y) > 0
  }
  fn p_below_or_collinear(&self, x: i32, y: i32) -> bool {
    self.relative_slope(x, y) >= 0
  }
  fn p_above(&self, x: i32, y: i32) -> bool {
    self.relative_slope(x, y) < 0
  }
  fn p_above_or_collinear(&self, x: i32, y: i32) -> bool {
    self.relative_slope(x, y) <= 0
  }
  fn p_collinear(&self, x: i32, y: i32) -> bool {
    self.relative_slope(x, y) == 0
  }
  fn line_collinear(&self, line: Line) -> bool {
    self.p_collinear(line.xi, line.yi) && self.p_collinear(line.xf, line.yf)
  }
}
```

Wow, that's a whole lot of code to just blindly trust. Let's make sure that we
have all that right with a test function. We'll write an easy test to start.

```rust
#[test]
fn line_tests() {
  let line_a = Line::new(0, 0, 1, 1);
  let line_b = Line::new(2, 2, 3, 3);
  assert!(line_a.line_collinear(line_b));
}
```

It passes! Okay, let's add some more. Let's even add something that we expect to
fail and see if it does:

```rust
#[test]
fn line_tests() {
  let line_a = Line::new(0, 0, 1, 1);
  let line_b = Line::new(2, 2, 3, 3);
  assert!(line_a.line_collinear(line_b));

  assert!(line_a.p_collinear(0, 0));
  assert!(line_a.p_above_or_collinear(0, 0));
  assert!(line_a.p_above(0, 0));
}
```

And then we get:

```
---- ppfov::line_tests stdout ----
thread 'ppfov::line_tests' panicked at 'assertion failed: line_a.p_above(0, 0)', src\ppfov.rs:49:3
note: Run with `RUST_BACKTRACE=1` for a backtrace.
```

This is good! We obviously can't say that `line_a` is colinear with (0,0) **and
also** above (0,0), it can only be one or the other. We expected a failure and
we got it, so our intuition has worked out so far. Let's fix that and then add
some more. Also, if you're coding along with me, and if your editor is using the
[Rust Language Server](https://github.com/rust-lang-nursery/rls) to show
warnings as you go, you'll note that using a method in a test makes it so that
it's not marked as dead code by the rls. This is also good, now we know what
methods never get called by the test. Of course, the RLS is also broken half the
time, so you can't seriously rely on it for anything, but when it _does_ work
it's nice. Here's our final test:

```rust
#[test]
fn line_tests() {
  let line_a = Line::new(0, 0, 1, 1);
  let line_b = Line::new(2, 2, 3, 3);
  assert!(line_a.line_collinear(line_b));

  assert!(line_a.p_collinear(0, 0));
  assert!(line_a.p_above_or_collinear(0, 0));
  assert!(line_a.p_above(1, 0));

  assert!(line_a.p_below_or_collinear(0, 1));
  assert!(line_a.p_below(0, 1));
}
```

So, what can we say from this? Well, it seems like the "ordering" of the terms
in the method names that we took from the example code is a little reversed from
what we might expect:

![p-above](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week03-07.png)

Yeah... that's not quite how English works. English uses Subject Verb Object
word ordering, and you should only deviate from that if you really have a good
reason. In a thing where a lot of shorthand is already going on, we don't have a
good enough reason, and since we're doing a lot of geometry work without being
able to see anything as we're working, we want to be as clear as we can. So
we'll just flip those names around to make them easier to understand.

```rust
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
```

So, is `line_a` _above the point_ `(1,0)`? Of course it is! That's a lot easier
to think about. I think. Let's move on before I get too lost in my own dumb
words. Now that we can have a single sight line, we combine two sight lines and
their view bumps to make a View.

```rust
#[derive(Debug, Clone)]
struct View {
  shallow_line: Line,
  steep_line: Line,
  shallow_bumps: Vec<(i32, i32)>,
  steep_bumps: Vec<(i32, i32)>,
}
```

One thing to note here is that `Line` is `Copy`, but `View` is _not_. This is
because each view contains a `Vec` of all the past bumps, so if we want to
duplicate the `View` we have to duplicate the bump lists too. Making empty
vectors is essentially free, but making vectors with even one element means we
have to go do a heap allocation. Also, the case where we don't have any bumps
(which means we don't allocate for the `Vec`) is also the case where we have to
scan the most squares (because none of the views end early). So no matter how it
works out we'll be taking some pain with FOV.

Making a new view is easy:

```rust
impl View {
  fn new(shallow_line: Line, steep_line: Line) -> Self {
    Self {
      shallow_line,
      steep_line,
      shallow_bumps: vec![],
      steep_bumps: vec![],
    }
  }

  // ...
```

And adding a bump to a View is a little more complicated. We have to do that
stuff where we use the lists of old bumps to adjust things based on a diagram we
didn't see. So, we'll look at what the python code does and then blindly trust
it for now.

```rust
  // ...

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
```

Do you trust the code? You _shouldn't_. I sure don't trust it. Let's write
another test. Except this is some weird stuff for an algorithm that we've never
used before, so what's good test data? That's right, we'll use the diagrams that
we drew earlier. We can at least cover the cases for the diagrams that we do
have, even if we don't know how the bump lists work out.

```rust
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
```

It passes. That's not at all the best test coverage, but it's enough confidence
that we can move along with our code.

Now we've got to check a whole quadrant at once. This is where we're forced to
write something new that might look kind of weird, because we're going to write
our first [generic
function](https://doc.rust-lang.org/book/second-edition/ch10-00-generics.html).

```rust
fn check_quadrant<VB, VE>(
  visited: &mut HashSet<(i32, i32)>, start_x: i32, start_y: i32, dx: i32, dy: i32, extent_x: i32, extent_y: i32, vision_blocked: &VB,
  visit_effect: &mut VE,
) where
  VB: Fn(i32, i32) -> bool,
  VE: FnMut(i32, i32),
{
  debug_assert!(dx == -1 || dx == 1);
  debug_assert!(dy == -1 || dy == 1);
  debug_assert!(extent_x > 0);
  debug_assert!(extent_y > 0);
  unimplemented!();
}
```

Does that make sense? We're accepting a bunch of values:

* The set of tiles we've visited so far. We need this because we don't know if
  we're in q1 or not, and tiles along the x-axis and y-axis relative to the
  origin of the FOV will end up counting as being in more than one quadrant. To
  avoid them being visited more than once, we have to track tiles between
  quadrants.
* A starting x and starting y. These are the origin for this FOV call.
* A delta x and delta y, which signal which quadrant this `check_quadrant` call
  is doing. If the values aren't exactly either -1 of 1 then something is very
  wrong, so we'll even `debug_assert!` on it. Since we're the only ones who will
  call this function, not any code outside this module, we can have it be just a
  debug assert instead of a full assert.
* An extent x and extent y, which define how far from the FOV origin we can
  scan. We'll use this to set the far bounds of the initial shallow line and
  steep line, and so our math will depend on them being some sort of positive
  value, so we'll also debug assert on that for now.
* `vision_blocked` is a new looking thing, it's a `&VB`, and VB is defined at
  the end as being `Fn(i32,i32) -> bool`, which means any function that takes
  two `i32` values and gives a bool. In this case, an `x` and a `y`, and you get
  if vision is blocked or not. There's probably some sort of comparison to the
  terrain or something, but we don't care, because that's not our concern, so
  we're just generic over that operation.
* `visit_effect` is a `&mut VE`, with `VB` being defined as `FnMut(i32,i32)`, so
  it takes an `x` and a `y` and then "marks that tile as visited" in some way.
  Again we don't care on the details, so we're just generic over however that
  happens.

Now we add a little more to the middle.

```rust
{
  debug_assert!(dx == -1 || dx == 1);
  debug_assert!(dy == -1 || dy == 1);
  debug_assert!(extent_x > 0);
  debug_assert!(extent_y > 0);

  let shallow_line = Line::new(0, 1, extent_x, 0);
  let steep_line = Line::new(1, 0, 0, extent_y);
  let mut active_views = vec![View::new(shallow_line, steep_line)];

  unimplemented!();
}
```

Exactly like the diagram we had with the red and blue initial lines. Now we
just... do that scanning thing...

```rust
  for i in 1..=(extent_x + extent_y) {
    for j in (i - extent_x).max(0)..=i.min(extent_y) {
      if active_views.is_empty() {
        return;
      } else {
        let x = i - j;
        let y = j;
        unimplemented!("visit_coord");
      }
    }
  }
```

So what does `visit_coord` need? Well, almost everything we had passed to
`check_quadrant`. We don't need extent x and extent y, but we do need to add in
the offset x and offset y, along with the list of active views. Since we've got
a thing for a direction on x and y, and a thing for an offset on x and y, we'll
make `dx` and `dy` be `dir_x` and `dir_y`, so that it's clearer (since `dx`
would usually mean "delta x" on its own, which would be direction and offset in
a single value). So it has an outline like this:

```rust
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

  unimplemented!()
}
```

And then `check_quadrant` ends up looking like this:

```rust
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
```

One thing you might ask: "do we really want debug asserts for the same thing in
both places?" to which I say "hell yes we do." A debug assert is _free_ in the
final product (you just turn them off and it compiles right out), so any time
you know something about the data that you can definitely debug assert on, just
do it. If the assertion throws a panic you didn't expect at some random point
later in development, that means you're using the code in some strange new way.
And _maybe_ that's okay to do, and you can relax or remove the debug assertion,
but you probably want to be notified when you're using the code in some new
weird way, so that you can go back and check that everything is still fine.

So now we find the right View that this coordinate is part of.

```rust
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
```

So... what the frickity frack? Well we pick a top left and a bottom right. Easy.
And the minimum view index is 0 obviously, because indexes are `usize` values.
Now we do a `loop`, because we have to keep going until either we find our
target or run out of views to look at. Okay so far? Now, for each pass of the
`loop`, we call `active_views.get(view_index)`, which uses the
[get](https://doc.rust-lang.org/std/primitive.slice.html#method.get) method on
slices (a Vec will automatically
[Deref](https://doc.rust-lang.org/std/ops/trait.Deref.html) into a slice when
necessary), and then `match` on that. The `get` method will safely index into
the slice and give an `Option<&T>`: either `Some(val_ref)` if it's a legal
index, or `None` otherwise.

* If we get `None` we've gone past the end of the list without finding a result,
  so we return, because this location doesn't fit into any View.
* If we get `Some(view_ref)` we have another branch point:
  * First we check to see if the _steep line_ is below or collinear with the
    bottom right of this location. If that's the case, we're totally below the
    current location, so we add 1 to our `view_index` so that the next pass
    looks at the next view in the list (or gets a `None`).
  * Next we see if the _shallow line_ is above or collinear with the top left of
    this location. If that's the case then our view is totally above the
    location, but since we're always keeping our views sorted counter-clockwise
    in the list, that means we know that no view farther in the list will
    possibly have this location, so we return.
  * Finally, if both checks turned up `false`, then our current location is part
    of the current view, so we break out of the loop and keep going.

All on board? The next part is easy.

```rust
  let target = (start_x + (offset_x * dir_x), start_y + (offset_y * dir_y));
  if !visited.contains(&target) {
    visited.insert(target);
    visit_effect(target.0, target.1);
  }
```

We use our FOV origin, the offset from the FOV origin, and the direction of the
offset, to determine the "actual" location within the world space that we're
targeting. If that location isn't within the set of visited locations, we insert
it into the set and then apply the visit effect.

We're in the home stretch

```rust
  if vision_blocked(target.0, target.1) {
    unimplemented!()
  }
}
```

So _if and only if_ vision is blocked by this tile, we'll do some view updating.
Otherwise, we're already done. I know you're gonna love this last part.

```rust
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
```

Great, we have all the branches covered. Wait, what's `check_view` look like?
That one is super simple.

```rust
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
```

The `check_view` function just does that one extra special rule, where a view is
dead if the lines are the same and they pass through an extremity of the origin.

How does all this actually _finally_ get called? Well, first we start off with
some basic sanity checks for the FOV request as a whole, then we make a visited
set and pass it along with the correct quadrant info to four calls of
`check_quadrant`. It's wordy, but simple.

```rust
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
```

* "Hey, Lokathor, isn't it totally stupid that we're passing radius twice to
  each `check_quadrant` call?"
* "Yeah, sure, but like let's _turn it on_ first and see if the results look
  right before we go fiddling with too many of the particulars."

### Part 04d: We Turn It On

Okay so to turn it on, we just have to adjust the main method to call the FOV,
and then only draw if the location is within the FOV we saw.

```rust
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
      for (scr_x, scr_y, id_mut) in ids.iter_mut() {
        let loc_for_this_screen_position = Location {
          x: scr_x as i32,
          y: scr_y as i32,
        } + offset;
        if seen_set.contains(&(loc_for_this_screen_position.x, loc_for_this_screen_position.y)) {
          match game.creatures.get(&loc_for_this_screen_position) {
            Some(ref _creature) => {
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
        } else {
          *id_mut = b' ';
        }
      }
    }
```

Wow... that's some ugly stuff. We really wanna make that Location stuff have
some smoother transitions into and out of `(i32,i32)` tuples, or make the FOV
use Location directly, or something. Still, it compiles.

![it-turns-on](https://github.com/Lokathor/roguelike-tutorial-2018/blob/master/screenshots/week03-08.png)

And it works!

## Part 05: Placing Enemies

TODO
