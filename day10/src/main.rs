use std::collections::{HashMap, VecDeque, HashSet};
use geo::{Contains, Polygon, LineString, point};

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Coord {
    x: i64,
    y: i64
}

impl Coord {
    pub fn next_north(self) -> Self {
        Self { y: self.y - 1, ..self }
    }

    pub fn next_south(self) -> Self {
        Self { y: self.y + 1, ..self }
    }

    pub fn next_east(self) -> Self {
        Self { x: self.x + 1, ..self }
    }

    pub fn next_west(self) -> Self {
        Self { x: self.x - 1, ..self }
    }

    pub fn neighbours_for(self, pipe: PipeShape) -> Vec<Coord> {
        match pipe {
            PipeShape::VerticalPipe => vec![self.next_north(), self.next_south()],
            PipeShape::HorizontalPipe => vec![self.next_east(), self.next_west()],
            PipeShape::LPipe => vec![self.next_north(), self.next_east()],
            PipeShape::JPipe => vec![self.next_west(), self.next_north()],
            PipeShape::FPipe => vec![self.next_south(), self.next_east()],
            PipeShape::SevenPipe => vec![self.next_south(), self.next_west()],
        }
    }

}

impl From<(i64, i64)> for Coord {
    fn from((x, y): (i64, i64)) -> Self {
        Self { x, y }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum PipeShape {
    VerticalPipe,
    HorizontalPipe,
    LPipe,
    JPipe,
    FPipe,
    SevenPipe,
}

#[derive(Clone, Debug)]
pub struct Network {
    start_point: Coord,
    pipe_locations: HashMap<Coord, PipeShape>,
    max_size: Coord,
}

impl Network {
    pub fn find_loop_distances(&self) -> HashMap<Coord, i64> {
        let mut distances = HashMap::new();
        let mut coords_to_visit = VecDeque::new();
        let mut coords_visited = HashSet::new();
        coords_to_visit.push_back(self.start_point);
        distances.insert(self.start_point, 0);

        while let Some(coord) = coords_to_visit.pop_front() {
            // already calculated the shortest path from this one
            if coords_visited.contains(&coord) {
                continue;
            }

            // special case the start
            if coord == self.start_point {
                let all_neighbours = vec![coord.next_east(), coord.next_north(), coord.next_south(), coord.next_west()];
                for neighbour in all_neighbours {
                    if let Some(pipe) = self.pipe_locations.get(&neighbour) {
                        if neighbour.neighbours_for(*pipe).contains(&coord) {
                            // this neighbour does contain a pipe connecting to 'start'
                            // so it's part of the loop, ignore the others
                            coords_to_visit.push_back(neighbour);
                            distances.insert(neighbour, 1);
                        }
                    }
                }
            } else {
                // shortest path to here
                let my_distances = *distances.get(&coord).unwrap();
                // we're going through the loop so we must always be reaching an actual pipe
                // don't error check this
                for neighbour in coord.neighbours_for(*self.pipe_locations.get(&coord).unwrap()) {
                    if !coords_visited.contains(&neighbour) {
                        coords_to_visit.push_back(neighbour);
                        distances.insert(neighbour, my_distances + 1);
                    }
                }
            }

            coords_visited.insert(coord);
        }


        distances
    }

    pub fn find_ordered_loop(&self) -> HashMap<Coord, i64> {
        let mut distances = HashMap::new();
        let mut coords_to_visit = VecDeque::new();
        let mut coords_visited = HashSet::new();
        coords_to_visit.push_back(self.start_point);
        distances.insert(self.start_point, 0);

        while let Some(coord) = coords_to_visit.pop_front() {
            // already calculated the shortest path from this one
            if coords_visited.contains(&coord) {
                continue;
            }

            // special case the start
            if coord == self.start_point {
                let all_neighbours = vec![coord.next_east(), coord.next_north(), coord.next_south(), coord.next_west()];
                for neighbour in all_neighbours {
                    if let Some(pipe) = self.pipe_locations.get(&neighbour) {
                        if neighbour.neighbours_for(*pipe).contains(&coord) {
                            // this neighbour does contain a pipe connecting to 'start'
                            // so it's part of the loop, ignore the others
                            coords_to_visit.push_back(neighbour);
                            distances.insert(neighbour, 1);
                            break;
                        }
                    }
                }
            } else {
                let my_distances = *distances.get(&coord).unwrap();
                // we're going through the loop so we must always be reaching an actual pipe
                // don't error check this
                for neighbour in coord.neighbours_for(*self.pipe_locations.get(&coord).unwrap()) {
                    if !coords_visited.contains(&neighbour) {
                        coords_to_visit.push_back(neighbour);
                        distances.insert(neighbour, my_distances + 1);
                        break;
                    }
                }
            }

            coords_visited.insert(coord);
        }

        distances
    }

    pub fn only_loop(&self) -> Network {
        let distances = self.find_loop_distances();
        let mut pipes_in_loop = self.pipe_locations.iter()
            .map(|(c, p)| (*c, *p))
            .filter(|(c, _)| distances.contains_key(&c))
            .collect::<HashMap<_, _>>();

        // insert the start location also
        let pipe_from_above = match pipes_in_loop.get(&self.start_point.next_north()) {
            Some(PipeShape::VerticalPipe) | Some(PipeShape::FPipe) | Some(PipeShape::SevenPipe) => true,
            _ => false,
        };
        let pipe_from_below = match pipes_in_loop.get(&self.start_point.next_south()) {
            Some(PipeShape::VerticalPipe) | Some(PipeShape::JPipe) | Some(PipeShape::LPipe) => true,
            _ => false,
        };
        let pipe_from_left = match pipes_in_loop.get(&self.start_point.next_west()) {
            Some(PipeShape::HorizontalPipe) | Some(PipeShape::FPipe) | Some(PipeShape::LPipe) => true,
            _ => false,
        };
        let pipe_from_right = match pipes_in_loop.get(&self.start_point.next_east()) {
            Some(PipeShape::HorizontalPipe) | Some(PipeShape::JPipe) | Some(PipeShape::SevenPipe) => true,
            _ => false,
        };

        match (pipe_from_above, pipe_from_below, pipe_from_left, pipe_from_right) {
            (true, true, false, false) => pipes_in_loop.insert(self.start_point, PipeShape::VerticalPipe),
            (true, false, true, false) => pipes_in_loop.insert(self.start_point, PipeShape::JPipe),
            (true, false, false, true) => pipes_in_loop.insert(self.start_point, PipeShape::LPipe),
            (false, true, true, false) => pipes_in_loop.insert(self.start_point, PipeShape::SevenPipe),
            (false, true, false, true) => pipes_in_loop.insert(self.start_point, PipeShape::FPipe),
            (false, false, true, true) => pipes_in_loop.insert(self.start_point, PipeShape::HorizontalPipe),
            _ => unreachable!(),
        };

        Network {
            start_point: self.start_point,
            max_size: self.max_size,
            pipe_locations: pipes_in_loop
        }
    }
}

impl std::fmt::Display for Network {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Coord { x: max_x, y: max_y} = self.max_size;
        for y in 0..=max_y {
            for x in 0..=max_x {
                match self.pipe_locations.get(&(x, y).into()) {
                    Some(PipeShape::HorizontalPipe) => write!(f, "-")?,
                    Some(PipeShape::VerticalPipe) => write!(f, "|")?,
                    Some(PipeShape::JPipe) => write!(f, "J")?,
                    Some(PipeShape::FPipe) => write!(f, "F")?,
                    Some(PipeShape::SevenPipe) => write!(f, "7")?,
                    Some(PipeShape::LPipe) => write!(f, "L")?,
                    None => write!(f, ".")?,
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

pub fn pipe_winding_number_upper(coord: Coord, ordered_loop: &HashMap<Coord, i64>) -> i64 {
    let current_number = *ordered_loop.get(&coord).unwrap();
    let number_above = *ordered_loop.get(&coord.next_north()).unwrap();
    if current_number == number_above + 1 {
        return -1;
    } else if current_number == number_above - 1 {
        return 1;
    } else if current_number == 0 {
        // wraparound?
        return -1;
    } else {
        return 1;
    }
}

pub fn pipe_winding_number_lower(coord: Coord, ordered_loop: &HashMap<Coord, i64>) -> i64 {
    let current_number = *ordered_loop.get(&coord).unwrap();
    let number_below = *ordered_loop.get(&coord.next_south()).unwrap();
    if current_number == number_below + 1 {
        return 1;
    } else if current_number == number_below - 1 {
        return -1;
    } else if current_number == 0 {
        // wraparound?
        return 1;
    } else {
        return -1;
    }
}

pub fn part_2(network: &Network) -> usize {
    let ordered_loop = network.find_ordered_loop();
    let network = network.only_loop();

    let mut coords_inside = HashSet::new();
    // now we have just the loop to worry about, and we have a way of ordering it.
    let Coord { x: max_x, y: max_y} = network.max_size;
    for y in 0..=max_y {
        let upper_pipes_in_row: Vec<_> = network.pipe_locations
        .iter().filter_map(|(c, p)| 
            ((*p == PipeShape::VerticalPipe || *p == PipeShape::JPipe || *p == PipeShape::LPipe)
                 && c.y == y).then_some(*c)).collect();

        let lower_pipes_in_row: Vec<_> = network.pipe_locations
        .iter().filter_map(|(c, p)| 
            ((*p == PipeShape::VerticalPipe || *p == PipeShape::SevenPipe || *p == PipeShape::FPipe)
                && c.y == y).then_some(*c)).collect();

        for x in 0..=max_x {
            let current_coord: Coord = (x as i64, y as i64).into();
            // println!("Considering coord {:?}", current_coord);
            if network.pipe_locations.contains_key(&current_coord) {
                // this is part of the network, don't check it
                continue;
            }

            // need to check two lines: the 'upper half' line, which intersects |, J, and L
            // and the 'lower half' line, which intersects |, 7, and F
            let upper_winding_number: i64 = upper_pipes_in_row.iter().filter(|c| c.x > x)
                .map(|p| pipe_winding_number_upper(*p, &ordered_loop)).sum();

            let lower_winding_number: i64 = lower_pipes_in_row.iter().filter(|c| c.x > x)
                .map(|p| pipe_winding_number_lower(*p, &ordered_loop)).sum();

            if upper_winding_number != 0 || lower_winding_number != 0 {
                // println!("Coord {:?} is inside", current_coord);
                coords_inside.insert(current_coord);
            }
        }
    }

    coords_inside.len()
}

pub fn part_2_geo(network: &Network) -> usize {
    let mut ordered_loop: Vec<_> = network.find_ordered_loop().into_iter().collect();
    // need this to fill in start
    let network = network.only_loop();
    ordered_loop.sort_by_key(|(_, n)| *n);
    ordered_loop.retain(|(c, _)|
        matches!(network.pipe_locations.get(c).expect(&format!("Nothing in map for {:?}", c).to_owned()),
        PipeShape::FPipe | PipeShape::JPipe | PipeShape::SevenPipe | PipeShape::LPipe));

    let segments = ordered_loop.into_iter().map(|(c, _)| (c.x, c.y)).collect::<Vec<_>>();

    let polygon = Polygon::new(
        LineString::from(segments),
        vec![]
    );


    let mut coords_inside = HashSet::new();
    // now we have just the loop to worry about, and we have a way of ordering it.
    let Coord { x: max_x, y: max_y} = network.max_size;
    for y in 0..=max_y {
        for x in 0..=max_x {
            let point = point!(x: x, y: y);
            if polygon.contains(&point) {
                coords_inside.insert(point);
            }
        }
    }

    coords_inside.len()
}

pub fn parse_input(input: &str) -> Network {
    let mut pipe_locations = HashMap::new();
    let mut start_point = None;
    let mut max_size = None;
    for (y, line) in input.lines().enumerate() {
        for (x, c) in line.chars().enumerate() {
            let current_coord: Coord = (x as i64, y as i64).into();
            match c {
                '|' => { pipe_locations.insert(current_coord, PipeShape::VerticalPipe); },
                '-' => { pipe_locations.insert(current_coord, PipeShape::HorizontalPipe); },
                'F' => { pipe_locations.insert(current_coord, PipeShape::FPipe); },
                'J' => { pipe_locations.insert(current_coord, PipeShape::JPipe); },
                '7' => { pipe_locations.insert(current_coord, PipeShape::SevenPipe); },
                'L' => { pipe_locations.insert(current_coord, PipeShape::LPipe); },
                'S' => { start_point = Some(current_coord); },
                _ => { },
            }

            max_size = Some(current_coord.into());
        }
    }

    let start_point = start_point.unwrap();
    let max_size = max_size.unwrap();
    // probably need to figure out what the pipe at the start point _would_ be, I think

    Network {
        start_point, pipe_locations, max_size
    }
}

pub fn part_1(network: &Network) -> i64 {
    *network.find_loop_distances().values().max().unwrap()
}

fn main() {
    let input = include_str!("../input.txt");
    let network = parse_input(input);
    println!("Part 1: {}", part_1(&network));
    println!("Part 2: {}", part_2(&network));
    println!("Part 2 geo: {}", part_2_geo(&network));
}

#[test]
pub fn test_simple_loop() {
    let input = r".....
.S-7.
.|.|.
.L-J.
.....";

    let network = parse_input(input);
    assert_eq!(part_1(&network), 4);
}

#[test]
pub fn test_complex_loop() {
    let input = r"..F7.
.FJ|.
SJ.L7
|F--J
LJ...";
    let network = parse_input(input);
    assert_eq!(part_1(&network), 8);
}

#[test]
pub fn test_part2() {
    let input = r"...........
.S-------7.
.|F-----7|.
.||.....||.
.||.....||.
.|L-7.F-J|.
.|..|.|..|.
.L--J.L--J.
...........";

    let network = parse_input(input);
    assert_eq!(part_2(&network), 4);
}

#[test]
pub fn test_part2_slim() {
    let input = r"..........
.S------7.
.|F----7|.
.||....||.
.||....||.
.|L-7F-J|.
.|..||..|.
.L--JL--J.
..........";

    let network = parse_input(input);
    assert_eq!(part_2(&network), 4);
}


#[test]
pub fn test_part2_large() {
    let input = r".F----7F7F7F7F-7....
.|F--7||||||||FJ....
.||.FJ||||||||L7....
FJL7L7LJLJ||LJ.L-7..
L--J.L7...LJS7F-7L7.
....F-J..F7FJ|L7L7L7
....L7.F7||L7|.L7L7|
.....|FJLJ|FJ|F7|.LJ
....FJL-7.||.||||...
....L---J.LJ.LJLJ...";

    let network = parse_input(input);
    assert_eq!(part_2(&network), 8);
}

#[test]
pub fn test_part2_final() {
    let input = r"FF7FSF7F7F7F7F7F---7
L|LJ||||||||||||F--J
FL-7LJLJ||||||LJL-77
F--JF--7||LJLJ7F7FJ-
L---JF-JLJ.||-FJLJJ7
|F|F-JF---7F7-L7L|7|
|FFJF7L7F-JF7|JL---7
7-L-JL7||F7|L7F-7F7|
L.L7LFJ|||||FJL7||LJ
L7JLJL-JLJLJL--JLJ.L";

    let network = parse_input(input);
    assert_eq!(part_2(&network), 10);
    dbg!(part_2_geo(&network));
}

