use std::collections::{HashSet, BTreeSet};

use itertools::Itertools;

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
pub struct Coord {
    x: i64,
    y: i64,
}

impl Coord {
    pub fn taxicab_distance_to(&self, other: Coord) -> u64 {
        self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    }
}

impl From<(i64, i64)> for Coord {
    fn from((x, y): (i64, i64)) -> Self {
        Coord { x, y }
    }
}

#[derive(Clone, Debug)]
pub struct Input {
    galaxies: HashSet<Coord>,
    blank_x: BTreeSet<i64>,
    blank_y: BTreeSet<i64>,
}

impl Input {
    pub fn adjusted_distance_between(&self, first: Coord, second: Coord, expansion_factor: usize) -> usize {
        let naive_distance = first.taxicab_distance_to(second);
        let blank_x_between = self.blank_x.range(first.x.min(second.x) .. first.x.max(second.x)).count();
        let blank_y_between = self.blank_y.range(first.y.min(second.y) .. first.y.max(second.y)).count();
        naive_distance as usize + (blank_x_between * (expansion_factor - 1)) + (blank_y_between * (expansion_factor - 1))
    }
}

pub fn parse_input(input: &str) -> Input {
    let mut galaxies: HashSet<Coord> = HashSet::new();
    let mut blank_x = BTreeSet::new();
    let mut blank_y = BTreeSet::new();
    let mut max_size = None;

    for (y, line) in input.lines().enumerate() {
        for (x, c) in line.chars().enumerate() {
            let current_coord = (x as i64, y as i64).into();
            if c == '#' {
                galaxies.insert(current_coord);
            }

            max_size = Some(current_coord);
        }
    }

    let max_size = max_size.unwrap();
    for y in 0..=max_size.y {
        if galaxies.iter().filter(|c| c.y == y).next().is_none() {
            blank_y.insert(y);
        }
    }

    for x in 0..=max_size.x {
        if galaxies.iter().filter(|c| c.x == x).next().is_none() {
            blank_x.insert(x);
        }
    }

    Input { galaxies, blank_x, blank_y }
}

pub fn solve(input: &Input, expansion_factor: usize) -> usize {
    input.galaxies.iter()
        .tuple_combinations()
        .map(|(f, s)| input.adjusted_distance_between(*f, *s, expansion_factor))
        .sum()
}

fn main() {
    let input = include_str!("../input.txt");
    let input = parse_input(input);
    println!("Part 1: {}", solve(&input, 2));
    println!("Part 1: {}", solve(&input, 1_000_000));
}

#[test]
pub fn test() {
    let input = r"...#......
.......#..
#.........
..........
......#...
.#........
.........#
..........
.......#..
#...#.....";

    let input = dbg!(parse_input(input));
    assert_eq!(solve(&input, 2), 374);
    assert_eq!(solve(&input, 10), 1030);
    assert_eq!(solve(&input, 100), 8410);
}