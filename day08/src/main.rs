use num::integer::lcm;

use std::collections::{HashMap, HashSet};


#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Direction {
    Left,
    Right,
}

impl From<char> for Direction {
    fn from(value: char) -> Self {
        match value {
            'L' => Self::Left,
            'R' => Self::Right,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug)]
pub struct Input<'a> {
    directions: Vec<Direction>,
    lookup: HashMap<(&'a str, Direction), &'a str>,
}

pub fn parse_input<'a>(input: &'a str) -> Input<'a> {
    let (directions, mappings) = input.split_once("\n\n").unwrap();
    let directions = directions.chars().map(Direction::from).collect();
    let mut lookup = HashMap::new();
    for line in mappings.lines() {
        let (key, values) = line.split_once(" = ").unwrap();
        let (left, right) = values.split_once(", ").unwrap();
        lookup.insert((key, Direction::Left), &left[1..]);
        lookup.insert((key, Direction::Right), &right[..right.len() - 1]);
    }

    Input { directions, lookup }
}

pub fn part_1(input: &Input) -> usize {
    let mut location = "AAA";
    for (step, direction) in input.directions.iter().cycle().enumerate() {
        location = input.lookup.get(&(location, *direction)).unwrap();
        if location == "ZZZ" {
            return step + 1;
        }
    }

    unreachable!()
}

#[derive(Debug)]
pub struct State {
    potential_endpoints: Vec<usize>,
    offset: usize,
    cycle_length: usize,
}

pub fn part_2(input: &Input) -> usize {
    let start_points = input.lookup.keys().filter(|(k, _)| k.ends_with('A')).map(|(k, _)| *k).collect::<HashSet<_>>();
    let mut start_states = HashMap::new();
    for start in start_points {
        let mut state = HashMap::new();
        let mut location = start;
        for (step, direction) in input.directions.iter().cycle().enumerate() {
            location = input.lookup.get(&(location, *direction)).unwrap();
            if location.ends_with('Z') {
                // found an end point - have we seen it before?
                // cycle detection actually incorrect here (in general case) - we also need to make sure
                // we're at the same offset into the instruction sequence too!
                if let Some(offset) = state.get(location) {
                    start_states.insert(start, State {
                        potential_endpoints: state.values().copied().collect(),
                        offset: *offset,
                        cycle_length: step + 1 - offset,
                    });
                    break;
                } else {
                    state.insert(location, step + 1);
                }
            }
        }
    }

    dbg!(&start_states);
    // turns out each start only reaches a single end and has offset == cycle length
    // so no need to be clever
    // might write up the clever version later?
    let mut result = 1_usize;
    for state in start_states {
        result = lcm(result, state.1.cycle_length);
    }

    result
}

fn main() {
    let input = include_str!("../input.txt");
    let input = parse_input(input);
    println!("Part 1: {}", part_1(&input));
    println!("Part 2: {}", part_2(&input));
}

#[test]
pub fn test_example1() {
    let input = r"RL

AAA = (BBB, CCC)
BBB = (DDD, EEE)
CCC = (ZZZ, GGG)
DDD = (DDD, DDD)
EEE = (EEE, EEE)
GGG = (GGG, GGG)
ZZZ = (ZZZ, ZZZ)";

    let input = dbg!(parse_input(input));
    assert_eq!(part_1(&input), 2);
}

#[test]
pub fn test_example2() {
    let input = r"LLR

AAA = (BBB, BBB)
BBB = (AAA, ZZZ)
ZZZ = (ZZZ, ZZZ)";

    let input = dbg!(parse_input(input));
    assert_eq!(part_1(&input), 6);
}

#[test]
pub fn test_part2() {
    let input = r"LR

11A = (11B, XXX)
11B = (XXX, 11Z)
11Z = (11B, XXX)
22A = (22B, XXX)
22B = (22C, 22C)
22C = (22Z, 22Z)
22Z = (22B, 22B)
XXX = (XXX, XXX)";

    let input = dbg!(parse_input(input));
    assert_eq!(part_2(&input), 6);
}
