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

/*
#[bench]
fn bench_step4_recur(b: &mut Bencher) {
  let gen = &mut PCG32::new(u64_from_time());
  b.iter(|| step4_recur(gen, 20));
}
*/
