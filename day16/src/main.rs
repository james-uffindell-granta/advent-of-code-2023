use std::collections::{HashMap, VecDeque, HashSet};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Coord {
    x: i64,
    y: i64,
}

impl Coord {
    pub fn next_up(self) -> Coord {
        Coord { y: self.y - 1, ..self }
    }

    pub fn next_left(self) -> Coord {
        Coord { x: self.x - 1, ..self }
    }

    pub fn next_down(self) -> Coord {
        Coord { y: self.y + 1, ..self }
    }

    pub fn next_right(self) -> Coord {
        Coord { x: self.x + 1, ..self }
    }
}

impl From<(i64, i64)> for Coord {
    fn from((x, y): (i64, i64)) -> Self {
        Self { x, y }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum Direction {
    Up, Down, Left, Right
}

#[derive(Debug, Clone)]
pub struct Cavern {
    tiles: HashMap<Coord, char>,
    max_size: Coord,
}

pub fn parse_input(input: &str) -> Cavern {
    let mut max_size = None;
    let mut tiles = HashMap::new();
    for (y, line) in input.lines().enumerate() {
        for (x, c) in line.chars().enumerate() {
            let current_coord: Coord = (x as i64, y as i64).into();
            if c != '.' {
                tiles.insert(current_coord, c);
            }

            max_size = Some(current_coord);
        }
    }

    let max_size = max_size.unwrap();
    Cavern { tiles, max_size }
}

pub fn get_energised_cells(cavern: &Cavern, start_state: (Coord, Direction)) -> usize {
    // which coords have had passing through them in which way
    let mut seen_states = HashMap::new();
    let mut cells_to_process = VecDeque::new();
    cells_to_process.push_back(start_state);

    while let Some((coord, direction)) = cells_to_process.pop_front() {
        if seen_states.insert((coord, direction), ()).is_some() {
            continue;
        }

        // ignore anything out of bounds as well
        if coord.x < 0 || coord.y < 0
            || coord.x > cavern.max_size.x || coord.y > cavern.max_size.y {
                continue;
            }

        match (cavern.tiles.get(&coord), direction) {
            (None | Some('|'), Direction::Up) => {
                cells_to_process.push_back((coord.next_up(), Direction::Up));
            },
            (None | Some('-'), Direction::Right) => {
                cells_to_process.push_back((coord.next_right(), Direction::Right));
            },
            (None | Some('-'), Direction::Left) => {
                cells_to_process.push_back((coord.next_left(), Direction::Left));
            },
            (None | Some('|'), Direction::Down) => {
                cells_to_process.push_back((coord.next_down(), Direction::Down));
            },
            (Some('\\'), Direction::Left) | (Some('/'), Direction::Right) => {
                cells_to_process.push_back((coord.next_up(), Direction::Up));
            },
            (Some('\\'), Direction::Right) | (Some('/'), Direction::Left) => {
                cells_to_process.push_back((coord.next_down(), Direction::Down));
            },
            (Some('\\'), Direction::Up) | (Some('/'), Direction::Down) => {
                cells_to_process.push_back((coord.next_left(), Direction::Left));
            },
            (Some('\\'), Direction::Down) | (Some('/'), Direction::Up) => {
                cells_to_process.push_back((coord.next_right(), Direction::Right));
            },
            (Some('|'), Direction::Left | Direction::Right) => {
                cells_to_process.push_back((coord.next_up(), Direction::Up));
                cells_to_process.push_back((coord.next_down(), Direction::Down));
            },
            (Some('-'), Direction::Up | Direction::Down) => {
                cells_to_process.push_back((coord.next_left(), Direction::Left));
                cells_to_process.push_back((coord.next_right(), Direction::Right));
            },
            _ => unreachable!(),
        }
    }

    let energised_coords = seen_states.keys().map(|(c, _)| c).collect::<HashSet<_>>();

    let mut energised_cells = 0_usize;
    for x in 0 ..= cavern.max_size.x {
        for y in 0 ..= cavern.max_size.y {
            let c = Coord::from((x, y));
            if energised_coords.contains(&c) {
                energised_cells += 1;
            }
        }
    }

    energised_cells
}

pub fn part_1(cavern: &Cavern) -> usize {
    get_energised_cells(cavern, ((0, 0).into(), Direction::Right))
}

pub fn part_2(cavern: &Cavern) -> usize {
    let best_result_from_top =
        (0 ..= cavern.max_size.x)
            .map(|x| ((x, 0).into(), Direction::Down))
            .map(|s| get_energised_cells(cavern, s))
            .max().unwrap();
    let best_result_from_bottom =
        (0 ..= cavern.max_size.x)
            .map(|x| ((x, cavern.max_size.y).into(), Direction::Up))
            .map(|s| get_energised_cells(cavern, s))
            .max().unwrap();
    let best_result_from_left =
        (0 ..= cavern.max_size.y)
            .map(|y| ((0, y).into(), Direction::Right))
            .map(|s| get_energised_cells(cavern, s))
            .max().unwrap();
    let best_result_from_right =
        (0 ..= cavern.max_size.y)
            .map(|y| ((cavern.max_size.x, y).into(), Direction::Left))
            .map(|s| get_energised_cells(cavern, s))
            .max().unwrap();

    best_result_from_top
        .max(best_result_from_bottom)
        .max(best_result_from_left)
        .max(best_result_from_right)
}

pub fn main() {
    let input = include_str!("../input.txt");
    let cavern = parse_input(input);
    println!("Part 1: {}", part_1(&cavern));
    println!("Part 2: {}", part_2(&cavern));
}

#[test]
pub fn test() {
    let input = r".|...\....
|.-.\.....
.....|-...
........|.
..........
.........\
..../.\\..
.-.-/..|..
.|....-|.\
..//.|....";

    let cavern = parse_input(input);
    assert_eq!(part_1(&cavern), 46);
    assert_eq!(part_2(&cavern), 51);
}