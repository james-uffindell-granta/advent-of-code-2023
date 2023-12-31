use std::{collections::{HashMap, HashSet, BTreeSet}, ops::Add};
use itertools::Itertools;

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug, Ord, PartialOrd)]
pub struct Coord {
    x: i64,
    y: i64,
}

impl Coord {
    pub fn next(self, direction: Direction) -> Coord {
        match direction {
            Direction::Up => self + (0, -1),
            Direction::Down => self + (0, 1),
            Direction::Left => self + (-1, 0),
            Direction::Right => self + (1, 0),
        }
    }

    pub fn previous(self, direction: Direction) -> Coord {
        match direction {
            Direction::Up => self + (0, 1),
            Direction::Down => self + (0, -1),
            Direction::Left => self + (1, 0),
            Direction::Right => self + (-1, 0),
        }
    }
}

impl From<(i64, i64)> for Coord {
    fn from((x, y): (i64, i64)) -> Self {
        Coord { x, y }
    }
}

impl Add<(i64, i64)> for Coord {
    type Output = Coord;

    fn add(self, (x, y): (i64, i64)) -> Self::Output {
        (self.x + x, self.y + y).into()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Direction {
    Up, Down, Left, Right,
}

impl Direction {
    pub fn heading(self) -> Heading {
        match self {
            Self::Up | Self::Down  => Heading::Vertical,
            Self::Left | Self::Right => Heading::Horizontal,
        }
    }

    pub fn possible_options(self) -> [Self; 2] {
        match self {
            Self::Up | Self::Down  => [Self::Left, Self::Right],
            Self::Left | Self::Right => [Self::Up, Self::Down],
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Heading {
    Horizontal, Vertical
}

impl Heading {
    pub fn possible_directions(self) -> [Direction; 2] {
        match self {
            Self::Vertical  => [Direction::Left, Direction::Right],
            Self::Horizontal => [Direction::Up, Direction::Down],
        }
    }
}

#[derive(Debug, Clone)]
pub struct City {
    block_weights: HashMap<Coord, u64>,
    max_size: Coord,
}

impl City {
    pub fn in_bounds(&self, c: Coord) -> bool {
        c.x >= 0 && c.x <= self.max_size.x && c.y >= 0 && c.y <= self.max_size.y
    }

    pub fn calculate_best_weights(&self, min_run: u64, max_run: u64) -> HashMap<(Coord, Heading), u64> {
        let all_headings = [Heading::Horizontal, Heading::Vertical];

        // to cope with the "at most three in a line", rather than calculating the best route
        // to a block as normal, we'll calculate "best route to a block that enters it heading in
        // direction D", for all directions that make sense for the block.
        // and consider all blocks in a line of min_run..=max_run to be equally 'neighbours'

        // keep track of which blocks we've finished with, and what our 'best route' numbers are
        let mut visited_blocks = HashSet::new();
        let mut best_routes = HashMap::new();
        // keep track of unfinished stuff in two ways: once in a map for easy lookup of current best cost,
        // and once in a set for easy retrieval of "smallest cost block" to handle next
        let mut unvisited_blocks = HashMap::new();
        let mut unvisited_blocks_sorted = BTreeSet::new();

        // fill in the unvisited blocks (do we actually need to do this?):
        for coord in self.block_weights.keys() {
            for heading in all_headings {
                unvisited_blocks.insert((*coord, heading), None);
            }
        }

        // replace the start point so we know we can get there (with either heading) in 0
        unvisited_blocks.insert((Coord::from((0, 0)), Heading::Horizontal), Some(0));
        unvisited_blocks.insert((Coord::from((0, 0)), Heading::Vertical), Some(0));
        unvisited_blocks_sorted.insert((0, Coord::from((0, 0)), Heading::Horizontal));
        unvisited_blocks_sorted.insert((0, Coord::from((0, 0)), Heading::Vertical));

        while let Some((score, coord, current_heading)) = unvisited_blocks_sorted.pop_first()
            {
                // bail out early condition - we've found the shortest way of getting there with some heading
                if coord == self.max_size {
                    best_routes.insert((coord, current_heading), score);
                    break;
                }

                if visited_blocks.contains(&(coord, current_heading)) {
                    unreachable!(); // just in case?
                    // continue;
                }

                // find the neighbours: this is all blocks within three of our current cell,
                // except in the direction we came from
                // (by assumption, we've exhausted that heading for this route)
                for direction in current_heading.possible_directions() {
                    let mut accumulated_loss_this_heading = 0;
                    let mut destination = coord;
                    for run in 1 ..= max_run {
                        destination = destination.next(direction);
                        match self.block_weights.get(&destination) {
                            Some(loss) => {
                                accumulated_loss_this_heading += loss;
                                let total_loss_here = score + accumulated_loss_this_heading;
                                if run < min_run {
                                    // not allowed to stop yet though
                                    continue;
                                }
                                let new_heading = direction.heading();
                                let current = unvisited_blocks.get(&(destination, new_heading));
                                match current {
                                    // we've found a better route
                                    Some(&Some(cost)) if cost > total_loss_here =>
                                        {
                                            unvisited_blocks.insert((destination, new_heading), Some(total_loss_here));
                                            unvisited_blocks_sorted.remove(&(cost, destination, new_heading));
                                            unvisited_blocks_sorted.insert((total_loss_here, destination, new_heading));

                                        },
                                    Some(None) => 
                                        {
                                            unvisited_blocks.insert((destination, new_heading), Some(total_loss_here));
                                            unvisited_blocks_sorted.insert((total_loss_here, destination, new_heading));
                                        },
                                    _ => { },
                                }
                            },
                            None => {
                                // shouldn't go this way anyway
                                break;
                            }
                        }
                    }
                }

                // mark this as visited and carry on
                visited_blocks.insert((coord, current_heading));
                // and also remove it from the unvisited list
                let best_score = unvisited_blocks.remove(&(coord, current_heading)).unwrap();
                best_routes.insert((coord, current_heading), best_score.unwrap());
            }

        best_routes
    }
}

pub fn parse_input(input: &str) -> City {
    let mut block_weights = HashMap::new();
    let mut max_size = None;

    for (y, line) in input.lines().enumerate() {
        for (x, c) in line.chars().enumerate() {
            let current_coord = (x as i64, y as i64).into();
            block_weights.insert(current_coord, c.to_digit(10).unwrap() as u64);
            max_size = Some(current_coord);
        }
    }

    let max_size = max_size.unwrap();
    City { block_weights, max_size }
}

pub fn part_1(city: &City) -> u64 {
    let distances = city.calculate_best_weights(1, 3);
    distances.into_iter().filter(|((c, _), _)| *c == city.max_size)
        .map(|(_, s)| s).min().unwrap()
}

pub fn part_2(city: &City) -> u64 {
    let distances = city.calculate_best_weights(4, 10);
    distances.into_iter().filter(|((c, _), _)| *c == city.max_size)
        .map(|(_, s)| s).min().unwrap()
}

fn main() {
    let input = include_str!("../input.txt");
    let city = parse_input(input);
    println!("Part 1: {}", part_1(&city));
    println!("Part 2: {}", part_2(&city));
}

#[test]
pub fn test() {
    let input = r"2413432311323
3215453535623
3255245654254
3446585845452
4546657867536
1438598798454
4457876987766
3637877979653
4654967986887
4564679986453
1224686865563
2546548887735
4322674655533";

    let city = parse_input(input);
    assert_eq!(part_1(&city), 102);
    assert_eq!(part_2(&city), 94);
}

#[test]
pub fn test_smaller() {
    let input = r"24
32";

    let city = parse_input(input);
    // dbg!(city.calculate_best_weights());
    // dbg!(part_1(&city));
}