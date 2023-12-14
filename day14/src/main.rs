use std::collections::{HashSet, HashMap, BTreeSet};

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug, Ord, PartialOrd)]
pub struct Coord {
    x: i64,
    y: i64,
}

impl Coord {
    // pub fn taxicab_distance_to(&self, other: Coord) -> u64 {
    //     self.x.abs_diff(other.x) + self.y.abs_diff(other.y)
    // }
}

impl From<(i64, i64)> for Coord {
    fn from((x, y): (i64, i64)) -> Self {
        Coord { x, y }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Direction {
    North, West, South, East,
}

#[derive(Debug, Clone)]
pub struct Input {
    cube_rocks: HashSet<Coord>,
    round_rocks: BTreeSet<Coord>,
    max_size: Coord,
}

impl Input {
    pub fn tilt(self, direction: Direction) -> Input {
        // cube rocks don't move, only round rocks do.
        let mut new_round_rocks: BTreeSet<Coord> = BTreeSet::new();
        let Coord { x: max_x, y: max_y} = self.max_size;
        match direction {
            Direction::North => {
                for y in 0 ..= max_y {
                    for x in 0 ..= max_x {
                        let current_coord = (x, y).into();
                        if self.round_rocks.contains(&current_coord) {
                            let nearest_cube_rock = self.cube_rocks.iter()
                                .filter_map(|c| (c.x == x && c.y < y).then_some(c.y))
                                .max().unwrap_or(-1);
                            let nearest_new_round_rock = new_round_rocks.iter()
                                .filter_map(|c| (c.x == x && c.y < y).then_some(c.y))
                                .max().unwrap_or(-1);
                            new_round_rocks.insert((x, nearest_cube_rock.max(nearest_new_round_rock) + 1).into());
                        }
                    }
                }
            },
            Direction::West => {
                for x in 0 ..= max_x {
                    for y in 0 ..= max_y {
                        let current_coord = (x, y).into();
                        if self.round_rocks.contains(&current_coord) {
                            let nearest_cube_rock = self.cube_rocks.iter()
                                .filter_map(|c| (c.y == y && c.x < x).then_some(c.x))
                                .max().unwrap_or(-1);
                            let nearest_new_round_rock = new_round_rocks.iter()
                                .filter_map(|c| (c.y == y && c.x < x).then_some(c.x))
                                .max().unwrap_or(-1);
                            new_round_rocks.insert((nearest_cube_rock.max(nearest_new_round_rock) + 1, y).into());
                        }
                    }
                }
            },
            Direction::South => {
                for y in (0 ..= max_y).rev() {
                    for x in 0 ..= max_x {
                        let current_coord = (x, y).into();
                        if self.round_rocks.contains(&current_coord) {
                            let nearest_cube_rock = self.cube_rocks.iter()
                                .filter_map(|c| (c.x == x && c.y > y).then_some(c.y))
                                .min().unwrap_or(self.max_size.y + 1);
                            let nearest_new_round_rock = new_round_rocks.iter()
                                .filter_map(|c| (c.x == x && c.y > y).then_some(c.y))
                                .min().unwrap_or(self.max_size.y + 1);
                            new_round_rocks.insert((x, nearest_cube_rock.min(nearest_new_round_rock) - 1).into());
                        }
                    }
                }
            },
            Direction::East => {
                for x in (0 ..= max_x).rev() {
                    for y in 0 ..= max_y {
                        let current_coord = (x, y).into();
                        if self.round_rocks.contains(&current_coord) {
                            let nearest_cube_rock = self.cube_rocks.iter()
                                .filter_map(|c| (c.y == y && c.x > x).then_some(c.x))
                                .min().unwrap_or(self.max_size.x + 1);
                            let nearest_new_round_rock = new_round_rocks.iter()
                                .filter_map(|c| (c.y == y && c.x > x).then_some(c.x))
                                .min().unwrap_or(self.max_size.x + 1);
                            new_round_rocks.insert((nearest_cube_rock.min(nearest_new_round_rock) - 1, y).into());
                        }
                    }
                }
            },
        }

        Input { round_rocks: new_round_rocks, ..self }
    }

    pub fn cycle(self) -> Input {
        self
            .tilt(Direction::North)
            .tilt(Direction::West)
            .tilt(Direction::South)
            .tilt(Direction::East)

    }

    pub fn north_weight(&self) -> i64 {
        // the weight of a round rock is max_size.y + 1 - y height?
        self.round_rocks.iter().map(|c| self.max_size.y + 1 - c.y).sum()
    }
}

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Coord { x: max_x, y: max_y} = self.max_size;
        for y in 0 ..= max_y {
            for x in 0 ..= max_x {
                let current_coord = (x, y).into();
                if self.cube_rocks.contains(&current_coord) {
                    write!(f, "#")?;
                } else if self.round_rocks.contains(&current_coord) {
                    write!(f, "O")?;
                } else {
                    write!(f, ".")?;
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}


pub fn parse_input(input: &str) -> Input {
    let mut round_rocks: BTreeSet<Coord> = BTreeSet::new();
    let mut cube_rocks: HashSet<Coord> = HashSet::new();
    let mut max_size = None;

    for (y, line) in input.lines().enumerate() {
        for (x, c) in line.chars().enumerate() {
            let current_coord = (x as i64, y as i64).into();
            if c == '#' {
                cube_rocks.insert(current_coord);
            } else if c == 'O' {
                round_rocks.insert(current_coord);
            }

            max_size = Some(current_coord);
        }
    }

    let max_size = max_size.unwrap();

    Input { round_rocks, cube_rocks, max_size }
}

pub fn part_1(input: &Input) -> i64 {
    input.clone().tilt(Direction::North).north_weight()
}

pub fn find_cycle(input: &Input) -> (Input, i64) {
    let number_of_cycles_to_run = 1_000_000_000;
    let mut states = HashMap::new();
    let mut state = input.clone();
    for cycle in 1 ..= number_of_cycles_to_run {
        state = state.cycle();
        if let Some(previous_cycle) = states.insert(state.round_rocks.clone(), cycle) {
            println!("Found state after cycle {} that matches cycle {}", cycle, previous_cycle);

            let cycle_length = cycle - previous_cycle;
            let remaining_period_to_fill = number_of_cycles_to_run - previous_cycle;
            let number_to_run_after_cycle = remaining_period_to_fill % cycle_length;

            return (state, number_to_run_after_cycle);
        }
    }

    unreachable!();
}

pub fn part_2(input: &Input) -> i64 {
    let (repeated_state, remaining_cycles) = find_cycle(input);
    let mut state = repeated_state;
    for _ in 1 ..= remaining_cycles {
        state = state.cycle();
    }

    state.north_weight()
}

fn main() {
    let input = include_str!("../input.txt");
    let input = parse_input(input);
    println!("Part 1: {}", part_1(&input));
    println!("Part 2: {}", part_2(&input));
}

#[test]
pub fn test() {
    let input = r"O....#....
O.OO#....#
.....##...
OO.#O....O
.O.....O#.
O.#..O.#.#
..O..#O..O
.......O..
#....###..
#OO..#....";
    let input = parse_input(input);
    assert_eq!(part_1(&input), 136);
}

#[test]
pub fn test_cycle() {
    let input = r"O....#....
O.OO#....#
.....##...
OO.#O....O
.O.....O#.
O.#..O.#.#
..O..#O..O
.......O..
#....###..
#OO..#....";
    let input = parse_input(input);
    // println!("{}", input);
    // let cycle_1 = input.cycle();
    // println!("{}", cycle_1);
    // let cycle_2 = cycle_1.cycle();
    // println!("{}", cycle_2);
    // let cycle_3 = cycle_2.cycle();
    // println!("{}", cycle_3);
    assert_eq!(part_2(&input), 64);
}
