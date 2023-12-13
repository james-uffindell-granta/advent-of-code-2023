use std::collections::HashSet;
use itertools::Itertools;

#[derive(Copy, Clone, Hash, PartialEq, Eq, Debug)]
pub enum Ground { Ash, Rock, }

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
        Pattern((0 .. self.0[0].len())
            .map(|n| self.0.iter().map(|v| v[n]).collect())
            .collect())
    }

    // finds horizontal lines only (transpose to find the others)
    pub fn find_symmetry(&self) -> Option<usize> {
        // every row except the last one could be the start of a symmetry
        let mut potential_symmetries = (0..self.0.len() - 1).collect::<Vec<_>>();

        potential_symmetries.retain(|p| {
            (0 ..= *p).rev().zip(p + 1 .. self.0.len()).all(|(l, r)| self.0[l] == self.0[r])
        });

        // don't forget to adjust the indexes (we want to start at 1)
        // and take the first (only) one
        potential_symmetries.iter().map(|x| x + 1).next()
    }

    pub fn find_near_symmetry(&self) -> Option<usize> {
        let mut potential_near_symmetries = (0..self.0.len() - 1).collect::<Vec<_>>();

        // have we already fixed a smudge for the symmetry starting at this offset?
        let mut smudge_fixed = HashSet::new();

        potential_near_symmetries.retain(|p| {
            for (l, r) in (0 ..= *p).rev().zip(p + 1 .. self.0.len()) {
                match Self::pair_fixable(&self.0[l], &self.0[r]) {
                    PairMatchState::Identical => { },
                    PairMatchState::Fixable => {
                        // remember we can fix up a row from this start point
                        // but if we've already used our smudge up, we can't do it again
                        if !smudge_fixed.insert(*p) {
                            return false;
                        }
                    },
                    PairMatchState::TooDifferent => return false,
                }
            }

            true
        });

        // but then: we _also_ only want to retain the rows that needed a smudge to get fixed
        potential_near_symmetries.retain(|p| smudge_fixed.contains(p));

        // don't forget to adjust the indexes (we want to start at 1)
        potential_near_symmetries.iter().map(|x| x + 1).next()     
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

pub fn solve(patterns: &[Pattern], selector: fn(&Pattern) -> Option<usize>) -> usize {
    patterns.iter().map(|p|
        match (selector(p), selector(&p.transpose())) {
            (Some(h), None) => 100 * h,
            (None, Some(v)) => v,
            _ => unreachable!(),
        }
    ).sum()
}

pub fn part_1(patterns: &[Pattern]) -> usize {
    solve(patterns, Pattern::find_symmetry)
}

pub fn part_2(patterns: &[Pattern]) -> usize {
    solve(patterns, Pattern::find_near_symmetry)
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
    assert_eq!(part_1(&patterns), 405);
    assert_eq!(part_2(&patterns), 400);
}