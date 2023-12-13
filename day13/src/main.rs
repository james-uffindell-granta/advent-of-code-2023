
use std::collections::{HashMap, HashSet};

use itertools::Itertools;

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub enum Ground {
    Ash, Rock,
}

impl From<char> for Ground {
    fn from(value: char) -> Self {
        match value {
            '#' => Self::Rock,
            '.' => Self::Ash,
            _ => unreachable!(),
        }
    }
}

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub enum PairMatchState {
    Identical,
    Fixable,
    TooDifferent,
}

#[derive(Clone, Debug)]
pub struct Pattern(Vec<Vec<Ground>>);

impl Pattern {
    pub fn transpose(&self) -> Pattern {
        Pattern((0..self.0[0].len())
            .map(|n| self.0.iter().map(|v| v[n]).collect())
            .collect())
    }

    pub fn find_symmetries(&self) -> Vec<usize> {
        let mut potential_symmetries = Vec::new();
        for (zero_based_line_number, (first, second)) in self.0.iter().tuple_windows().enumerate() {
            if first == second {
                potential_symmetries.push(zero_based_line_number);
            }
        }

        potential_symmetries.retain(|p| {
            (0 ..= *p).rev().zip(p + 1 .. self.0.len()).all(|(l, r)| self.0[l] == self.0[r])
        });

        // don't forget to adjust the indexes (we want to start at 1)
        potential_symmetries.iter().map(|x| x + 1).collect()
    }

    pub fn find_near_symmetries(&self) -> Vec<usize> {
        let mut potential_near_symmetries = Vec::new();

        // have we already fixed a smudge for the symmetry starting at this offset?
        let mut smudge_fixed = HashSet::new();

        for (zero_based_line_number, (first, second)) in self.0.iter().tuple_windows().enumerate() {
            match Self::pair_fixable(first, second) {
                PairMatchState::Identical => { potential_near_symmetries.push(zero_based_line_number); },
                PairMatchState::Fixable => {
                    // don't need to remember we fixed it up here - we'll track that later
                    potential_near_symmetries.push(zero_based_line_number);
                    
                }
                PairMatchState::TooDifferent => { },
            }
        }

        potential_near_symmetries.retain(|p| {
            for (l, r) in (0 ..= *p).rev().zip(p + 1 .. self.0.len()) {
                match Self::pair_fixable(&self.0[l], &self.0[r]) {
                    PairMatchState::Identical => continue,
                    PairMatchState::Fixable => {
                        // have we already fixed our smudge for this start point?
                        if smudge_fixed.get(p).is_some() {
                            return false;
                        }

                        // otherwise we can use our smudge to fix this pair and carry on
                        smudge_fixed.insert(*p);
                        continue
                    },
                    PairMatchState::TooDifferent => return false,
                }
            }

            true
        });

        // but then: we _also_ only want to retain the rows that needed a smudge to get fixed
        potential_near_symmetries.retain(|p| smudge_fixed.contains(p));

        // don't forget to adjust the indexes (we want to start at 1)
        potential_near_symmetries.iter().map(|x| x + 1).collect()        
    }

    // the pair of rows is fixable if there's only one component that differs
    pub fn pair_fixable<T>(first: &[T], second: &[T]) -> PairMatchState 
        where T : Eq {
            match first.iter().zip(second.iter()).filter(|(f, s)| f != s).count() {
                0 => PairMatchState::Identical,
                1 => PairMatchState::Fixable,
                _ => PairMatchState::TooDifferent,
            }
    }
}

pub fn parse_input(input: &str) -> Vec<Pattern> {
    let mut result = Vec::new();
    let patterns = input.split("\n\n");
    for pattern in patterns {
        result.push(Pattern(
            pattern.lines().map(|line| line.chars().map(Ground::from).collect()).collect()
        ));
    }

    result
}

pub fn part_1(patterns: &[Pattern]) -> usize {
    let mut total = 0;
    for pattern in patterns {
        // check for rows first
        let horizontal_lines = pattern.find_symmetries();
        let vertical_lines = pattern.transpose().find_symmetries();
        assert!(horizontal_lines.len() + vertical_lines.len() == 1);

        if horizontal_lines.is_empty() {
            total += vertical_lines[0];
        } else {
            total += 100 * horizontal_lines[0];
        }
    }

    total
}

pub fn part_2(patterns: &[Pattern]) -> usize {
    let mut total = 0;
    for pattern in patterns {
        // check for rows first
        let horizontal_lines = pattern.find_near_symmetries();
        let vertical_lines = pattern.transpose().find_near_symmetries();
        assert!(horizontal_lines.len() + vertical_lines.len() == 1);

        if horizontal_lines.is_empty() {
            total += vertical_lines[0];
        } else {
            total += 100 * horizontal_lines[0];
        }
    }

    total
}

fn main() {
    let input = include_str!("../input.txt");
    let patterns = parse_input(input);
    println!("Part 1: {}", part_1(&patterns));
    println!("Part 2: {}", part_2(&patterns));
}

#[test]
pub fn test() {
    let input = r"#.##..##.
..#.##.#.
##......#
##......#
..#.##.#.
..##..##.
#.#.##.#.

#...##..#
#....#..#
..##..###
#####.##.
#####.##.
..##..###
#....#..#";

    let patterns = parse_input(input);
    dbg!(patterns[0].find_symmetries());
    dbg!(patterns[0].transpose().find_symmetries());
    dbg!(patterns[1].find_symmetries());
    dbg!(patterns[1].transpose().find_symmetries());

    assert_eq!(part_1(&patterns), 405);
    assert_eq!(part_2(&patterns), 400);
}
