#![feature(int_roundings)]

use std::collections::BTreeMap;
use std::time::Instant;
use std::{collections::{HashSet, HashMap, BTreeSet}, ops::Add};

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug, Ord, PartialOrd)]
pub struct Coord {
    x: i64,
    y: i64,
}

impl Coord {
    pub fn neighbours(self) -> [Self; 4] {
        [ self + (0, 1),
        self + (1, 0),
        self + (0, -1),
        self + (-1, 0),]
    }

    // pub fn next(self, direction: Direction) -> Coord {
    //     match direction {
    //         Direction::Up => self + (0, -1),
    //         Direction::Down => self + (0, 1),
    //         Direction::Left => self + (-1, 0),
    //         Direction::Right => self + (1, 0),
    //     }
    // }
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

// #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
// pub enum Direction {
//     North, West, South, East,
// }

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug, Ord, PartialOrd)]
pub struct MetaCoord(Coord);

#[derive(Debug, Clone)]
pub struct Input {
    rocks: BTreeSet<Coord>,
    max_size: Coord,
    start_point: Coord,
}

impl Input {
    pub fn in_bounds(&self, c: Coord) -> bool {
        c.x >= 0 && c.x <= self.max_size.x && c.y >= 0 && c.y <= self.max_size.y
    }

    pub fn in_meta_bounds(&self, c: Coord, meta_c: MetaCoord) -> bool {
        let real_c = c + (- meta_c.0.x * (self.max_size.x + 1), - meta_c.0.y * (self.max_size.y + 1));
        self.in_bounds(real_c)
    }

    pub fn is_rock(&self, c: Coord, meta_c: MetaCoord) -> bool {
        let real_c = c + (- meta_c.0.x * (self.max_size.x + 1), - meta_c.0.y * (self.max_size.y + 1));
        self.rocks.contains(&real_c)
    }

    pub fn is_perimeter(&self, c: Coord, meta_c: MetaCoord) -> bool {
        let real_c = c + (- meta_c.0.x * (self.max_size.x + 1), - meta_c.0.y * (self.max_size.y + 1));
        real_c.x == 0 || real_c.x == self.max_size.x || real_c.y == 0 && real_c.y == self.max_size.y
    }

    pub fn get_meta_coord(&self, c: Coord) -> MetaCoord {
        MetaCoord((c.x.div_floor(self.max_size.x + 1), c.y.div_floor(self.max_size.y + 1)).into())
    }

    pub fn get_perimeter_facing(&self, within: MetaCoord, facing: MetaCoord) -> BTreeSet<Coord> {
        match (facing.0.x - within.0.x, facing.0.y - within.0.y) {
            (1, 0) => {
                let edge_x = within.0.x * (self.max_size.x + 1) + self.max_size.x;
                let top_y = within.0.y * (self.max_size.y + 1);
                let bottom_y = within.0.y * (self.max_size.y + 1) + self.max_size.y;
                (top_y ..= bottom_y).map(|y| (edge_x, y).into()).collect()
            },
            (-1, 0) => {
                let edge_x = within.0.x * (self.max_size.x + 1);
                let top_y = within.0.y * (self.max_size.y + 1);
                let bottom_y = within.0.y * (self.max_size.y + 1) + self.max_size.y;
                (top_y ..= bottom_y).map(|y| (edge_x, y).into()).collect()
            },
            (0, 1) => {
                let edge_y = within.0.y * (self.max_size.y + 1) + self.max_size.y;
                let left_x = within.0.x * (self.max_size.x + 1);
                let right_x = within.0.x * (self.max_size.x + 1) + self.max_size.x;
                (left_x ..= right_x).map(|x| (x, edge_y).into()).collect()
            },
            (0, -1) => {
                let edge_y = within.0.y * (self.max_size.y + 1);
                let left_x = within.0.x * (self.max_size.x + 1);
                let right_x = within.0.x * (self.max_size.x + 1) + self.max_size.x;
                (left_x ..= right_x).map(|x| (x, edge_y).into()).collect()
            },
            _ => unreachable!(),
        }
    }

    pub fn get_equivalent_real_coord(&self, c: Coord, meta_c: MetaCoord) -> Coord {
        c + (- meta_c.0.x * (self.max_size.x + 1), - meta_c.0.y * (self.max_size.y + 1))
    }

    pub fn get_equivalent_coord_in_meta(&self, c: Coord, meta_c: MetaCoord) -> Coord {
        let current_meta_coord = self.get_meta_coord(c);
        let real_coord = self.get_equivalent_real_coord(c, current_meta_coord);
        real_coord + (meta_c.0.x * (self.max_size.x + 1), meta_c.0.y * (self.max_size.y + 1))
    }

    pub fn calculate_weights(&self) -> HashMap<Coord, u64> {

        // keep track of which blocks we've finished with, and what our 'best route' numbers are
        let mut visited_blocks = HashSet::new();
        let mut best_routes = HashMap::new();
        // keep track of unfinished stuff in two ways: once in a map for easy lookup of current best cost,
        // and once in a set for easy retrieval of "smallest cost block" to handle next
        let mut unvisited_blocks = HashMap::new();
        let mut unvisited_blocks_sorted = BTreeSet::new();

        // fill in the unvisited blocks (do we actually need to do this?):
        // for y in 0 ..= self.max_size.y {
        //     for x in 0 ..= self.max_size.x {
        //         let c = Coord::from((x, y));
        //         unvisited_blocks.insert(c, None);
        //     }
        // }

        unvisited_blocks.insert(self.start_point, Some(0));
        unvisited_blocks_sorted.insert((0, self.start_point));

        while let Some((score, coord)) = unvisited_blocks_sorted.pop_first()
        {
            // bail out early condition - we've found the shortest way of getting there with some heading
            // if coord == self.max_size {
            //     best_routes.insert((coord, current_heading), score);
            //     break;
            // }

            if visited_blocks.contains(&coord) {
                // unreachable!(); // just in case?
                continue;
            }

            // find the neighbours: this is all blocks within three of our current cell,
            // except in the direction we came from
            // (by assumption, we've exhausted that heading for this route)
            for neighbour in coord.neighbours() {
                // println!("Currently coord's neighbour is: {:?}", neighbour);
                if self.rocks.contains(&neighbour) {
                    // println!("This is a rock, can't go there");
                    // can't go here
                    continue;
                }

                if !self.in_bounds(neighbour) {
                    // println!("This is out of bounds, can't go there");
                    // gone outside
                    continue;
                }

                let current_weight = unvisited_blocks.get(&neighbour);
                match current_weight {
                    Some(&Some(weight)) if weight > score + 1 => {
                        // found a better route
                        // println!("Seen this neighbour before but {} is better than {}", score + 1, weight);
                        unvisited_blocks.insert(neighbour, Some(score + 1));
                        unvisited_blocks_sorted.remove(&(weight, neighbour));
                        unvisited_blocks_sorted.insert((score + 1, neighbour));
                    },
                    None | Some(None) => {
                        // println!("Haven't seen this neighbour before");
                        unvisited_blocks.insert(neighbour, Some(score + 1));
                        unvisited_blocks_sorted.insert((score + 1, neighbour));
                    },
                    _ => {
                        // println!("Current weight for this neighbour is {:?}", current_weight);
                        // todo should this be possible?
                    }
                }
            }

            // mark this as visited and carry on
            visited_blocks.insert(coord);
            // and also remove it from the unvisited list
            let best_score = unvisited_blocks.remove(&coord).unwrap();
            // println!("Remembering that we've handled coord {:?} with score {:?}", coord, best_score);
            best_routes.insert(coord, best_score.unwrap());
        }

        best_routes
    }

    // fills in one copy of the repeating pattern given a set of starting weights for some coords.
    pub fn calculate_weights_from(&self, meta: MetaCoord, known_weights: &HashMap<Coord, u64>) -> HashMap<Coord, u64> {
        // keep track of which blocks we've finished with, and what our 'best route' numbers are
        let mut visited_blocks = HashSet::new();
        let mut best_routes = HashMap::new();
        // keep track of unfinished stuff in two ways: once in a map for easy lookup of current best cost,
        // and once in a set for easy retrieval of "smallest cost block" to handle next
        let mut unvisited_blocks = HashMap::new();
        let mut unvisited_blocks_sorted = BTreeSet::new();

        for (coord, weight) in known_weights {
            unvisited_blocks.insert(*coord, Some(*weight));
            unvisited_blocks_sorted.insert((*weight, *coord));
        }

        while let Some((score, coord)) = unvisited_blocks_sorted.pop_first()
        {
            if visited_blocks.contains(&coord) {
                // unreachable!(); // just in case?
                continue;
            }

            // find the neighbours: this is all blocks within three of our current cell,
            // except in the direction we came from
            // (by assumption, we've exhausted that heading for this route)
            for neighbour in coord.neighbours() {
                // println!("Meta {:?}: currently coord's neighbour is: {:?}", meta, neighbour);
                if self.is_rock(neighbour, meta) {
                    // println!("This is a rock, can't go there");
                    // can't go here
                    continue;
                }

                if !self.in_meta_bounds(neighbour, meta) {
                    // println!("This is out of bounds, can't go there");
                    // gone outside
                    continue;
                }

                if visited_blocks.contains(&neighbour) {
                    continue;
                }

                let current_weight = unvisited_blocks.get(&neighbour);
                match current_weight {
                    Some(&Some(weight)) if weight > score + 1 => {
                        // found a better route
                        // println!("Seen this neighbour before but {} is better than {}", score + 1, weight);
                        unvisited_blocks.insert(neighbour, Some(score + 1));
                        unvisited_blocks_sorted.remove(&(weight, neighbour));
                        unvisited_blocks_sorted.insert((score + 1, neighbour));
                    },
                    None | Some(None) => {
                        // println!("Haven't seen this neighbour before");
                        unvisited_blocks.insert(neighbour, Some(score + 1));
                        unvisited_blocks_sorted.insert((score + 1, neighbour));
                    },
                    _ => {
                        // println!("Current weight for this neighbour is {:?}", current_weight);
                        // todo should this be possible?
                    }
                }
            }

            // mark this as visited and carry on
            visited_blocks.insert(coord);
            // and also remove it from the unvisited list
            let best_score = unvisited_blocks.remove(&coord).unwrap();
            // println!("Remembering that we've handled coord {:?} with score {:?}", coord, best_score);
            if self.in_meta_bounds(coord, meta) {
                best_routes.insert(coord, best_score.unwrap());
            }
        }

        best_routes
    }

    pub fn calculate_weights_extended(&self, threshold: u64) -> HashMap<MetaCoord, HashMap<Coord, u64>> {
        let mut seen_states: HashMap<BTreeMap<Coord, u64>, HashMap<Coord, u64>> = HashMap::new();
        let now = Instant::now();
        let mut num_cells_reachable = 0_usize;
        let mut visited_meta_blocks = HashSet::new();
        let mut meta_block_reachable_cell_counts = HashMap::new();

        // keep track of which blocks we've finished with, and what our 'best route' numbers are
        // let mut visited_blocks = HashSet::new();
        // let mut best_routes = HashMap::new();

        // keep track of unfinished stuff in two ways: once in a map for easy lookup of current best cost,
        // and once in a set for easy retrieval of "smallest cost block" to handle next
        let mut unvisited_meta_blocks = HashMap::new();
        let mut unvisited_meta_blocks_sorted = BTreeSet::new();

        let max_grid_dimension = (self.max_size.x + 1).max(self.max_size.y + 1) as u64;
        let max_possible_meta_distance_to_travel = threshold / max_grid_dimension + 1;
        
        let start_meta = MetaCoord((0, 0).into());
        unvisited_meta_blocks.insert(start_meta, Some(0));
        unvisited_meta_blocks_sorted.insert((0, start_meta));
        meta_block_reachable_cell_counts.insert(start_meta, self.calculate_weights_from(start_meta, &HashMap::from([(self.start_point, 0)])));

        while let Some((score, meta_coord)) = unvisited_meta_blocks_sorted.pop_first()
        {
            // TODO we need to do something here - how do we know if we've gone far enough?
            if score > max_possible_meta_distance_to_travel {
                // we've checked every meta-coord that would be within the range of travel
                break;
            }

            if meta_block_reachable_cell_counts.get(&meta_coord).unwrap()
                .iter().all(|(_, w)| *w > threshold) {
                // couldn't get anywhere better from here anyway
                continue;
            }

            if visited_meta_blocks.contains(&meta_coord) {
                // unreachable!(); // just in case?
                continue;
            }

            // find the neighbours: this is all blocks within three of our current cell,
            // except in the direction we came from
            // (by assumption, we've exhausted that heading for this route)
            for neighbour in meta_coord.0.neighbours().map(MetaCoord) {
                // println!("Considering neighbour {:?}", neighbour);

                if visited_meta_blocks.contains(&neighbour) {
                    // println!("Already handled this neighbour");
                    continue;
                }

                let current_weight = unvisited_meta_blocks.get(&neighbour);
                match current_weight {
                    Some(&Some(weight)) if weight > score => {

                        let perimeter_facing = self.get_perimeter_facing(meta_coord, neighbour);
                        let perimeter_weights = meta_block_reachable_cell_counts.get(&meta_coord).unwrap().iter().filter(|(c, _)| perimeter_facing.contains(c)).map(|(c, w)| (*c, *w)).collect::<HashMap<_, _>>();
                        if perimeter_weights.iter().all(|(_, w)| *w > threshold) {
                            // not even worth considering this neighbour
                            continue;
                        }

                        let min_perimeter_weight = perimeter_weights.values().copied().min().unwrap();
                        let normalised_perimeter_state = perimeter_weights.iter().map(|(c, w)| (self.get_equivalent_real_coord(*c, meta_coord), w - min_perimeter_weight)).collect::<BTreeMap<_, _>>();

                        let new_neighbour_weights = if let Some(seen_state) = seen_states.get(&normalised_perimeter_state) {
                            seen_state.iter().map(|(c, w)| (self.get_equivalent_coord_in_meta(*c, neighbour), w + min_perimeter_weight)).collect::<HashMap<_, _>>()
                        } else {
                            let new_neighbour_weights = self.calculate_weights_from(neighbour, &perimeter_weights);
                            seen_states.insert(normalised_perimeter_state, new_neighbour_weights.iter().map(|(c, w)| (*c, w - min_perimeter_weight)).collect::<HashMap<_, _>>());
                            new_neighbour_weights
                        };

                        // found a better route
                        // println!("Seen this neighbour before but {} is better than {}", score + 1, weight);
                        unvisited_meta_blocks.insert(neighbour, Some(score + 1));
                        unvisited_meta_blocks_sorted.remove(&(weight, neighbour));
                        unvisited_meta_blocks_sorted.insert((score + 1, neighbour));
                        // let new_neighbour_weights = self.calculate_weights_from(neighbour, &perimeter_weights);

                        let existing_neighbour_weights = meta_block_reachable_cell_counts.get(&neighbour).unwrap();
                        // must have exactly the same set of coords in it? all non-walls, surely
                        let mut adjusted_neighbour_weights = HashMap::new();
                        for (c, w) in new_neighbour_weights {
                            let existing_weight = existing_neighbour_weights.get(&c).unwrap();
                            adjusted_neighbour_weights.insert(c, w.min(*existing_weight));
                        }
                        meta_block_reachable_cell_counts.insert(neighbour, adjusted_neighbour_weights);
                    },
                    None | Some(None) => {
                        // we now also need to fill in a first pass at the neighbour's weights
                        let perimeter_facing = self.get_perimeter_facing(meta_coord, neighbour);
                        let perimeter_weights = meta_block_reachable_cell_counts.get(&meta_coord).unwrap().iter().filter(|(c, _)| perimeter_facing.contains(c)).map(|(c, w)| (*c, *w)).collect::<HashMap<_, _>>();
                        if perimeter_weights.iter().all(|(_, w)| *w > threshold) {
                            // not even worth considering this neighbour
                            continue;
                        }

                        let min_perimeter_weight = perimeter_weights.values().copied().min().unwrap();
                        let normalised_perimeter_state = perimeter_weights.iter().map(|(c, w)| (self.get_equivalent_real_coord(*c, meta_coord), w - min_perimeter_weight)).collect::<BTreeMap<_, _>>();

                        let neighbour_weights = if let Some(seen_state) = seen_states.get(&normalised_perimeter_state) {
                            seen_state.iter().map(|(c, w)| (self.get_equivalent_coord_in_meta(*c, neighbour), w + min_perimeter_weight)).collect::<HashMap<_, _>>()
                        } else {
                            let neighbour_weights = self.calculate_weights_from(neighbour, &perimeter_weights);
                            seen_states.insert(normalised_perimeter_state, neighbour_weights.iter().map(|(c, w)| (*c, w - min_perimeter_weight)).collect::<HashMap<_, _>>());
                            neighbour_weights
                        };

                        // println!("Haven't seen this neighbour before");
                        unvisited_meta_blocks.insert(neighbour, Some(score + 1));
                        unvisited_meta_blocks_sorted.insert((score + 1, neighbour));

                        // let neighbour_weights = self.calculate_weights_from(neighbour, &perimeter_weights);
                        // dbg!(&neighbour_weights);
                        meta_block_reachable_cell_counts.insert(neighbour, neighbour_weights);
                    },
                    _ => {
                        // should still be nothing to do - if we can get there in strictly fewer
                        // meta-steps, then the interior weights don't need updating
                    }
                }
            }

            // mark this as visited and carry on
            visited_meta_blocks.insert(meta_coord);
            // and also remove it from the unvisited list
            unvisited_meta_blocks.remove(&meta_coord).unwrap();
        }

        meta_block_reachable_cell_counts
    }
}

impl std::fmt::Display for Input {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let weights = self.calculate_weights();
        let Coord { x: max_x, y: max_y} = self.max_size;
        for y in 0 ..= max_y {
            for x in 0 ..= max_x {
                let current_coord = (x, y).into();
                if self.rocks.contains(&current_coord) {
                    write!(f, "  # ")?;
                } else {
                    let weight = weights.get(&current_coord).unwrap();
                    write!(f, "{: >4} ", weight)?;
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

pub fn parse_input(input: &str) -> Input {
    let mut rocks: BTreeSet<Coord> = BTreeSet::new();
    let mut max_size = None;
    let mut start_point = None;

    for (y, line) in input.lines().enumerate() {
        for (x, c) in line.chars().enumerate() {
            let current_coord = (x as i64, y as i64).into();
            if c == '#' {
                rocks.insert(current_coord);
            } else if c == 'S' {
                start_point = Some(current_coord);
            }

            max_size = Some(current_coord);
        }
    }

    let max_size = max_size.unwrap();
    let start_point = start_point.unwrap();

    Input { rocks, max_size, start_point }
}

pub fn part_1(input: &Input, distance: u64) -> usize {
    // let weights = dbg!(input.calculate_weights());
    let weights = input.calculate_weights_from(
        MetaCoord((0, 0).into()),
        &HashMap::from([(input.start_point, 0)]));
    weights.into_iter()
        .filter(|(_, w)| w <= &distance && w % 2 == distance % 2)
        .count()
}

pub fn part_2(input: &Input) -> usize {
    // for my input at least it's 131 by 131 and it costs 65 to get to each edge midpoint and 130 to get to each corner regardless of the placement of the blocks
    // because the perimeter and start row/column have no blocks in

    // the number of steps is 26,501,365 which is 202,300 * 131 + 65
    // so after we get to the midpoint of a side the furthest we can go is an additional 202,300 grids in that direction (and we can just reach the end of each of those)
    // so we can reach 202,299 grid spaces fully in each cardinal direction (all squares in those grid with the right parity), plus part of the 202,300th

    // because the grid size is odd, each time we move into an adjacent grid we swap from picking up the 'odd' cells (as in the initial square) to picking up the 'even' cells
    // there are 202,300^2 'even' grids we reach where we need to include the even squares (= 40,925,290,000)
    // and 202,299^2 'odd' grids we reach where we need to include the odd squares (= 40,924,885,401)

    // there are then 'inner diagonals', where we can reach everything but an outer corner - there are 202,299 of these in each quadrant.
    // and 'outer diagonals', where we can reach only one inner corner - there are 202,300 of these in each quadrant.
    // the four furthest squares on each compass point are each unique and should be fine from this fake version.

    // go two out in each direction to figure out the weightings - this should be enough to get us everything we need, and gives us the right parity on the partial grids on the diagonals.
    let fake_threshold = 2 * 131 + 65;
    let weights_of_expanded = input.calculate_weights_extended(fake_threshold);

    let weights = weights_of_expanded.iter().map(|(m, ws)| (m, ws.values().filter(|w| **w <= fake_threshold && *w % 2 == 1).count())).collect::<HashMap<_, _>>();

    // this is how many things can be reached in an 'odd' square
    let weights_of_center = weights.get(&MetaCoord((0, 0).into())).unwrap();
    // this is how many things can be reached in an 'even' square any of the four adjacent squares should be fine
    let weights_of_even_square = weights.get(&MetaCoord((0, 1).into())).unwrap();

    // answer is:
    (weights_of_center * 202_299 * 202_299)
    + (weights_of_even_square * 202_300 * 202_300)
    + weights.get(&MetaCoord((0, 2).into())).unwrap()
    + weights.get(&MetaCoord((0, -2).into())).unwrap()
    + weights.get(&MetaCoord((2, 0).into())).unwrap()
    + weights.get(&MetaCoord((-2, 0).into())).unwrap()
    + (weights.get(&MetaCoord((-1, -2).into())).unwrap() * 202_300)
    + (weights.get(&MetaCoord((1, -2).into())).unwrap() * 202_300)
    + (weights.get(&MetaCoord((1, 2).into())).unwrap() * 202_300)
    + (weights.get(&MetaCoord((-1, 2).into())).unwrap() * 202_300)
    + (weights.get(&MetaCoord((-1, 1).into())).unwrap() * 202_299)
    + (weights.get(&MetaCoord((-1, -1).into())).unwrap() * 202_299)
    + (weights.get(&MetaCoord((1, 1).into())).unwrap() * 202_299)
    + (weights.get(&MetaCoord((1, -1).into())).unwrap() * 202_299)

    // so that's 81,850,175,401 complete grids
    // let total_full_grids_reachable = 81_850_175_401_u64;
}

fn main() {
    let input = include_str!("../input.txt");
    let input = parse_input(input);
    println!("Part 1: {}", part_1(&input, 64));
    // println!("{}", input);
    println!("Part 2: {}", part_2(&input));
}

#[test]
pub fn test() {
    let input = r"...........
.....###.#.
.###.##..#.
..#.#...#..
....#.#....
.##..S####.
.##..#...#.
.......##..
.##.#.####.
.##..##.##.
...........";
    let input = parse_input(input);
    assert_eq!(part_1(&input, 6), 16);
    let now = Instant::now();
    dbg!(input.calculate_weights_extended(1000));
    println!("Took {:2?}", now.elapsed());
}
