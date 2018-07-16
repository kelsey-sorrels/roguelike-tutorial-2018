#![allow(non_upper_case_globals)]

use super::*;

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

/// Rolls a step roll, according to the 4th edition chart.
pub fn step(gen: &mut PCG32, mut step: i32) -> i32 {
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

#[derive(Debug, Clone)]
pub struct PCG32 {
  state: u64,
}

impl Default for PCG32 {
  /// Makes a generator with the default state suggested by Wikipedia.
  fn default() -> Self {
    PCG32 { state: 0x4d595df4d0f33173 }
  }
}

impl PCG32 {
  pub fn new(state: u64) -> Self {
    Self { state }
  }

  pub fn next_u32(&mut self) -> u32 {
    const A: u64 = 6364136223846793005;
    const C: u64 = 1442695040888963407; // this can be any odd const
    self.state = self.state.wrapping_mul(A).wrapping_add(C);
    let mut x = self.state;
    let rotation = (x >> 59) as u32;
    x ^= x >> 18;
    ((x >> 27) as u32).rotate_right(rotation)
  }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash)]
pub struct RandRangeInclusive32 {
  base: u32,
  width: u32,
  reject: u32,
}

impl RandRangeInclusive32 {
  pub fn new(range_incl: RangeInclusive<u32>) -> Self {
    let (low, high) = range_incl.into_inner();
    assert!(low < high, "RandRangeInclusive32 must go from low to high, got {} ..= {}", low, high);
    let base = low;
    let width = (high - low) + 1;
    debug_assert!(width > 0);
    let width_count = ::std::u32::MAX / width;
    let reject = (width_count * width) - 1;
    RandRangeInclusive32 { base, width, reject }
  }

  /// Lowest possible result of this range.
  pub fn low(&self) -> u32 {
    self.base
  }

  /// Highest possible result of this range.
  pub fn high(&self) -> u32 {
    self.base + (self.width - 1)
  }

  /// Converts any `u32` into `Some(val)` if the input can be evenly placed
  /// into range, or `None` otherwise.
  pub fn convert(&self, roll: u32) -> Option<u32> {
    if roll > self.reject {
      None
    } else {
      Some(self.base + (roll % self.width))
    }
  }

  pub fn roll_with(&self, gen: &mut PCG32) -> u32 {
    loop {
      if let Some(output) = self.convert(gen.next_u32()) {
        return output;
      }
    }
  }

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
}

/// Rolls a 1d4 when used.
pub const d4: RandRangeInclusive32 = RandRangeInclusive32 {
  base: 1,
  width: 4,
  reject: 4294967291,
};
#[test]
fn test_d4_const_is_correct() {
  assert_eq!(d4, RandRangeInclusive32::new(1..=4))
}
#[test]
#[ignore]
pub fn range_range_inclusive_32_sample_validity_test_d4() {
  let the_range = d4;
  let mut outputs: [u32; 5] = [0; 5];
  // 0 to one less than max
  for u in 0..::std::u32::MAX {
    let opt = the_range.convert(u);
    match opt {
      Some(roll) => outputs[roll as usize] += 1,
      None => outputs[0] += 1,
    };
  }
  // max
  let opt = the_range.convert(::std::u32::MAX);
  match opt {
    Some(roll) => outputs[roll as usize] += 1,
    None => outputs[0] += 1,
  };
  let mut output_iter = outputs.iter().cloned();
  let rejections = output_iter.next().unwrap();
  // non-power of 2 dice reject less than their width out of the full `u32`
  // range
  assert!(rejections == d4.width);
  let ones = output_iter.next().unwrap();
  while let Some(output) = output_iter.next() {
    assert_eq!(ones, output, "{:?}", outputs);
  }
}

/// Rolls a 1d6 when used.
pub const d6: RandRangeInclusive32 = RandRangeInclusive32 {
  base: 1,
  width: 6,
  reject: 4294967291,
};
#[test]
fn test_d6_const_is_correct() {
  assert_eq!(d6, RandRangeInclusive32::new(1..=6))
}
#[test]
#[ignore]
pub fn range_range_inclusive_32_sample_validity_test_d6() {
  let the_range = d6;
  let mut outputs: [u32; 7] = [0; 7];
  // 0 to one less than max
  for u in 0..::std::u32::MAX {
    let opt = the_range.convert(u);
    match opt {
      Some(roll) => outputs[roll as usize] += 1,
      None => outputs[0] += 1,
    };
  }
  // max
  let opt = the_range.convert(::std::u32::MAX);
  match opt {
    Some(roll) => outputs[roll as usize] += 1,
    None => outputs[0] += 1,
  };
  let mut output_iter = outputs.iter().cloned();
  let rejections = output_iter.next().unwrap();
  // non-power of 2 dice reject less than their width out of the full `u32`
  // range
  assert!(rejections < d6.width);
  let ones = output_iter.next().unwrap();
  while let Some(output) = output_iter.next() {
    assert_eq!(ones, output, "{:?}", outputs);
  }
}

/// Rolls a 1d8 when used.
pub const d8: RandRangeInclusive32 = RandRangeInclusive32 {
  base: 1,
  width: 8,
  reject: 4294967287,
};
#[test]
fn test_d8_const_is_correct() {
  assert_eq!(d8, RandRangeInclusive32::new(1..=8))
}
#[test]
#[ignore]
pub fn range_range_inclusive_32_sample_validity_test_d8() {
  let the_range = d8;
  let mut outputs: [u32; 9] = [0; 9];
  // 0 to one less than max
  for u in 0..::std::u32::MAX {
    let opt = the_range.convert(u);
    match opt {
      Some(roll) => outputs[roll as usize] += 1,
      None => outputs[0] += 1,
    };
  }
  // max
  let opt = the_range.convert(::std::u32::MAX);
  match opt {
    Some(roll) => outputs[roll as usize] += 1,
    None => outputs[0] += 1,
  };
  let mut output_iter = outputs.iter().cloned();
  let rejections = output_iter.next().unwrap();
  // non-power of 2 dice reject less than their width out of the full `u32`
  // range
  assert!(rejections == d8.width);
  let ones = output_iter.next().unwrap();
  while let Some(output) = output_iter.next() {
    assert_eq!(ones, output, "{:?}", outputs);
  }
}

/// Rolls a 1d10 when used.
pub const d10: RandRangeInclusive32 = RandRangeInclusive32 {
  base: 1,
  width: 10,
  reject: 4294967289,
};
#[test]
fn test_d10_const_is_correct() {
  assert_eq!(d10, RandRangeInclusive32::new(1..=10))
}
#[test]
#[ignore]
pub fn range_range_inclusive_32_sample_validity_test_d10() {
  let the_range = d10;
  let mut outputs: [u32; 10 + 1] = [0; 10 + 1];
  // 0 to one less than max
  for u in 0..::std::u32::MAX {
    let opt = the_range.convert(u);
    match opt {
      Some(roll) => outputs[roll as usize] += 1,
      None => outputs[0] += 1,
    };
  }
  // max
  let opt = the_range.convert(::std::u32::MAX);
  match opt {
    Some(roll) => outputs[roll as usize] += 1,
    None => outputs[0] += 1,
  };
  let mut output_iter = outputs.iter().cloned();
  let rejections = output_iter.next().unwrap();
  // non-power of 2 dice reject less than their width out of the full `u32`
  // range
  assert!(rejections < d10.width);
  let ones = output_iter.next().unwrap();
  while let Some(output) = output_iter.next() {
    assert_eq!(ones, output, "{:?}", outputs);
  }
}

/// Rolls a 1d12 when used.
pub const d12: RandRangeInclusive32 = RandRangeInclusive32 {
  base: 1,
  width: 12,
  reject: 4294967291,
};
#[test]
fn test_d12_const_is_correct() {
  assert_eq!(d12, RandRangeInclusive32::new(1..=12))
}
#[test]
#[ignore]
pub fn range_range_inclusive_32_sample_validity_test_d12() {
  let the_range = d12;
  let mut outputs: [u32; 12 + 1] = [0; 12 + 1];
  // 0 to one less than max
  for u in 0..::std::u32::MAX {
    let opt = the_range.convert(u);
    match opt {
      Some(roll) => outputs[roll as usize] += 1,
      None => outputs[0] += 1,
    };
  }
  // max
  let opt = the_range.convert(::std::u32::MAX);
  match opt {
    Some(roll) => outputs[roll as usize] += 1,
    None => outputs[0] += 1,
  };
  let mut output_iter = outputs.iter().cloned();
  let rejections = output_iter.next().unwrap();
  // non-power of 2 dice reject less than their width out of the full `u32`
  // range
  assert!(rejections < d12.width);
  let ones = output_iter.next().unwrap();
  while let Some(output) = output_iter.next() {
    assert_eq!(ones, output, "{:?}", outputs);
  }
}

/// Rolls a 1d20 when used.
pub const d20: RandRangeInclusive32 = RandRangeInclusive32 {
  base: 1,
  width: 20,
  reject: 4294967279,
};
#[test]
fn test_d20_const_is_correct() {
  assert_eq!(d20, RandRangeInclusive32::new(1..=20))
}
#[test]
#[ignore]
pub fn range_range_inclusive_32_sample_validity_test_d20() {
  let the_range = d20;
  let mut outputs: [u32; 20 + 1] = [0; 20 + 1];
  // 0 to one less than max
  for u in 0..::std::u32::MAX {
    let opt = the_range.convert(u);
    match opt {
      Some(roll) => outputs[roll as usize] += 1,
      None => outputs[0] += 1,
    };
  }
  // max
  let opt = the_range.convert(::std::u32::MAX);
  match opt {
    Some(roll) => outputs[roll as usize] += 1,
    None => outputs[0] += 1,
  };
  let mut output_iter = outputs.iter().cloned();
  let rejections = output_iter.next().unwrap();
  // non-power of 2 dice reject less than their width out of the full `u32`
  // range
  assert!(rejections < d20.width);
  let ones = output_iter.next().unwrap();
  while let Some(output) = output_iter.next() {
    assert_eq!(ones, output, "{:?}", outputs);
  }
}
