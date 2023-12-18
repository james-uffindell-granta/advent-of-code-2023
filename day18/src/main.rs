use std::{collections::{HashMap, HashSet, BTreeSet}, ops::Add};
use nom::{
    bytes::complete::tag,
    character::complete as cc,
    combinator::{all_consuming, map},
    sequence::{separated_pair, tuple},
    Finish,
    IResult,
};
use colored::*;
use itertools::Itertools;

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

    pub fn next(self, direction: Direction) -> Coord {
        match direction {
            Direction::Up => self + (0, -1),
            Direction::Down => self + (0, 1),
            Direction::Left => self + (-1, 0),
            Direction::Right => self + (1, 0),
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

impl From<char> for Direction {
    fn from(value: char) -> Self {
        match value {
            'U' => Self::Up,
            'L' => Self::Left,
            'D' => Self::Down,
            'R' => Self::Right,
            _ => unreachable!(),
        }
    }
}

#[derive(Debug, Clone)]
pub struct DigPlan {
    instructions: Vec<(Direction, u64, (u8, u8, u8))>,
}

impl DigPlan {
    pub fn to_lagoon(&self) -> Lagoon {
        let mut location: Coord = (0, 0).into();
        let mut walls = HashMap::new();
        let mut vertices = Vec::new();
        let mut vertical_segments = Vec::new();
        let mut horizontal_segments = Vec::new();
        vertices.push(location);
        walls.insert(location, None);
        for (direction, amount, color) in &self.instructions {
            let end_location = match direction {
                Direction::Up => location + (0, -(*amount as i64)),
                Direction::Down => location + (0, (*amount as i64)),
                Direction::Right => location + ((*amount as i64), 0),
                Direction::Left => location + (-(*amount as i64), 0),
            };
            match direction {
                Direction::Up => vertical_segments.push(VerticalSegment::from(location, end_location)),
                Direction::Down => vertical_segments.push(VerticalSegment::from(location, end_location)),
                Direction::Right => horizontal_segments.push(HorizontalSegment::from(location, end_location)),
                Direction::Left => horizontal_segments.push(HorizontalSegment::from(location, end_location)),
            }
            // for _ in 1 ..= *amount {
            //     location = location.next(*direction);
            //     walls.insert(location, Some(*color));
            // }
            location = end_location;
            vertices.push(location);
        }

        let min_x = vertices.iter().map(|c| c.x).min().unwrap();
        let min_y = vertices.iter().map(|c| c.y).min().unwrap();
        let max_x = vertices.iter().map(|c| c.x).max().unwrap();
        let max_y= vertices.iter().map(|c| c.y).max().unwrap();

        let bounds = ((min_x, min_y).into(), (max_x, max_y).into());

        Lagoon { walls, vertices, vertical_segments, horizontal_segments, bounds }
    }    
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct VerticalSegment {
    lower: Coord,
    upper: Coord,
}

impl VerticalSegment {
    pub fn from(start: Coord, end: Coord) -> VerticalSegment {
        assert!(start.x == end.x);
        VerticalSegment {
            lower: (start.x, start.y.min(end.y)).into(),
            upper: (start.x, start.y.max(end.y)).into(),
        }
    }

    pub fn intercept(&self, y: i64) -> Option<i64> {
        if y >= self.lower.y && y <= self.upper.y {
            Some(self.lower.x)
        } else {
            None
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct HorizontalSegment {
    lower: Coord,
    upper: Coord,
}

impl HorizontalSegment {
    pub fn from(start: Coord, end: Coord) -> Self {
        assert!(start.y == end.y);
        Self {
            lower: (start.x.min(end.x), start.y).into(),
            upper: (start.x.max(end.x), start.y).into(),
        }
    }

    pub fn intercept(&self, x: i64) -> Option<i64> {
        if x >= self.lower.x && x <= self.upper.x {
            Some(self.lower.y)
        } else {
            None
        }
    }
}

#[derive(Debug, Clone)]
pub struct Lagoon {
    walls: HashMap<Coord, Option<(u8, u8, u8)>>,
    vertices: Vec<Coord>,
    vertical_segments: Vec<VerticalSegment>,
    bounds: (Coord, Coord),
    horizontal_segments: Vec<HorizontalSegment>,
}

impl std::fmt::Display for Lagoon {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Coord { x: min_x, y: min_y} = self.bounds.0;
        let Coord { x: max_x, y: max_y} = self.bounds.1;
        for y in min_y ..= max_y {
            for x in min_x ..= max_x {
                let current_coord = (x, y).into();
                match self.walls.get(&current_coord) {
                    Some(Some((r, g, b))) => { write!(f, "{}", "#".truecolor(*r, *g, *b))?; },
                    Some(None) => { write!(f, "#")?; },
                    None => { write!(f, ".")?; },
                }
            }
            writeln!(f)?;
        }

        Ok(())
    }
}

impl Lagoon {
    pub fn find_outside(&self) -> HashSet<Coord> {
        // flood fill from perimeter
        let mut determined_exterior = HashSet::new();
        let mut potential_exterior = BTreeSet::new();
        for y in self.bounds.0.y ..= self.bounds.1.y {
            potential_exterior.insert(Coord::from((self.bounds.0.x, y)));
            potential_exterior.insert(Coord::from((self.bounds.1.x, y)));
        }

        for x in self.bounds.0.x ..= self.bounds.1.x {
            potential_exterior.insert(Coord::from((x, self.bounds.0.y)));
            potential_exterior.insert(Coord::from((x, self.bounds.1.y)));
        }

        while let Some(next) = potential_exterior.pop_first() {
            if self.walls.contains_key(&next) {
                continue;
            }

            if next.x < self.bounds.0.x || next.x > self.bounds.1.x
                || next.y < self.bounds.0.y || next.y > self.bounds.1.y {
                    // out of bounds
                    continue;
                }

            // otherwise this is 'outside'; check its neighbours, and skip any 'in-wall' ones
            for c in next.neighbours() {
                if !self.walls.contains_key(&c) && !determined_exterior.contains(&c) {
                    potential_exterior.insert(c);
                }
            }

            determined_exterior.insert(next);
        }

        determined_exterior
    }

    pub fn find_inside(&self) -> u64 {
        // flood fill from perimeter
        // let mut determined_interior = HashSet::new();
        // let mut potential_interior = BTreeSet::new();
        let mut interior_size = 0u64;
        'a: for y in self.bounds.0.y ..= self.bounds.1.y {
            let mut inside = false;
            let mut vertical_walls_intersecting = self.vertical_segments.iter().filter_map(|s| s.intercept(y)).collect::<BTreeSet<_>>();
            let vertices_in_row = self.vertices.iter().filter(|c| c.y == y).map(|c| c.x).collect::<HashSet<_>>();
            let vertical_segment_lower_vertices = self.vertical_segments.iter()
                .filter(|s| s.lower.y == y)
                .map(|s| s.lower.x).collect::<HashSet<_>>();
            let horizontal_segment_lower_vertices = self.horizontal_segments.iter()
                .filter(|s| s.lower.y == y)
                .map(|s| s.lower.x).collect::<HashSet<_>>();
            // let segment_upper_vertices = self.segments.iter().map(|s| s.upper.x).collect::<HashSet<_>>();
            // println!("Intersections in row {} are {:?}", y, vertical_walls_intersecting);
            // this adds up all the right bits "between" segments - not including the actual number of intersections.
            // for (first, second) in vertical_walls_intersecting.iter().tuple_windows() {
            //     if vertices_in_row.contains(first) && vertices_in_row.contains(second) {
            //         // both vertices: we have two options
            //         // either these are the start-end of a single line segment (in which case we add it regardless)
            //         // or this is the end-start of two separate line segments (in which case we add it conditionally)
            //         // this is a horizontal line segment
            //         // add it all to the inside
            //         // println!("{} and {} are both vertices", first, second);
            //         if horizontal_segment_lower_vertices.contains(first) {
            //             interior_size += (second - first - 1) as u64;
            //             // now we need to handle 'flipping'
            //             // because these two intercepts are logically 'the same line', if they go in opposite directions
            //             // then we've crossed it and need to change parity
            //             match (vertical_segment_lower_vertices.contains(first), vertical_segment_lower_vertices.contains(second)) {
            //                 // both vertices are the bottom or the top of a segment -
            //                 // so this doesn't change our inside/outside parity
            //                 (true, true) | (false, false) => {
            //                     // println!("Both intercepts were lower or upper");
            //                 },
            //                 // otherwise this was a horizontal jog in a vertical line
            //                 // so we have changed whether we're inside or outside
            //                 (true, false) | (false, true) => {
            //                     // println!("Inside was {} but is now {}", inside, !inside);
            //                     inside = !inside;
            //                 },
            //             }
            //         } else {
            //             // this is the end of one horizontal line and the start of another unrelated one
            //             if inside {
            //                 interior_size += (second - first - 1) as u64;
            //             }
            //             // these are unrelated lines so no 'flipping' happens
            //         }

                    
            //     } else if vertices_in_row.contains(first) {
            //         // the first component was a vertex but the second is a real wall
            //         // if we were inside before we still are, so add this in
            //         if inside {
            //             // but don't include the vertex because that's included in case 1
            //             interior_size += (second - first - 1) as u64;
            //         }
            //         // println!("{} was a vertex but {} wasn't", first, second);
            //         // println!("Inside was {} but is now {}", inside, !inside);
            //         // we're _about_ to cross a wall but we haven't yet
            //         inside = !inside;
            //     } else if vertices_in_row.contains(second) {
            //         // the first component is a wall but the second is a vertex
            //         // same as case 2 except
            //         // we need to have been outside before for this to count
            //         if !inside {
            //             interior_size += (second - first - 1) as u64;
            //         }
            //         // println!("{} wasn't a vertex but {} was", first, second);
            //         // println!("Inside was {} but is now {}", inside, !inside);
            //         inside = !inside;
            //     } else {
            //         // both components are vertical walls (not vertices)
            //         // this flips the parity twice so we only want to add on the interior if we were outside
            //         // to start with
            //         // println!("{} and {} are both not vertices; inside is {}", first, second, inside);
            //         if !inside {
            //             interior_size += (second - first - 1) as u64;
            //         }
            //     }
            // }
            
            interior_size += vertical_walls_intersecting.len() as u64;

            // println!("INTERIOR SIZE SO FAR: {}", interior_size);
            let mut inside = false;
            let mut last_wall = None;
            while let Some(intercept) = vertical_walls_intersecting.pop_first() {
                match last_wall {
                    // we're processing a wall but we have seen a wall before - should we add the space between
                    // that wall and us?
                    Some(wall) => {
                        if inside {
                            interior_size += (intercept - wall - 1) as u64;
                        }
                    },
                    None => {
                        // this must be the corner that starts the row
                    },
                }

                if vertices_in_row.contains(&intercept) {
                    // this must be the start of a horizontal line
                    // so the next intercept must be its end
                    let next = vertical_walls_intersecting.pop_first().unwrap();
                    assert!(vertices_in_row.contains(&next));

                    // include this many cells plus one
                    // we include this regardless of whether we were previously "inside" or "outside"
                    interior_size += (next - intercept - 1) as u64;

                    match (vertical_segment_lower_vertices.contains(&intercept), vertical_segment_lower_vertices.contains(&next)) {
                        // both vertices are the bottom or the top of a segment -
                        // so this doesn't change our inside/outside parity
                        (true, true) | (false, false) => {
                            // println!("Both intercepts were lower or upper");
                        },
                        // otherwise this was a horizontal jog in a vertical line
                        // so we have changed whether we're inside or outside
                        (true, false) | (false, true) => {
                            // println!("Inside was {} but is now {}", inside, !inside);
                            inside = !inside;
                        },
                    }

                    last_wall = Some(next);
                } else {
                    // this is a vertical wall going past us
                    inside = !inside;
                    last_wall = Some(intercept);
                }

                // otherwise this is an actual boundary

            }



            // let mut wall_state = WallStatus::OutsideWall;
            // let mut state = Status::Outside;
            // let mut last_coord = None;
            // let mut wall_coords = self.walls.keys().filter(|c| c.y == y).collect::<BTreeSet<_>>();

            // while let Some(wall) = wall_coords.pop_first() {
            //     match wall_state {
            //         // the last square was outside a wall - so now we've entered
            //         WallStatus::OutsideWall => { wall_state = WallStatus::JustEnteredWall(*wall); },
            //         // the last square we looked at was a wall with 'outside' on the left
            //         WallStatus::JustEnteringWall(c) => {
            //             match last_coord {
            //                 // was it literally the square to our left? if so we're going along a horizontal wall
            //                 Some(c) if c == wall.next(Direction::Left) => {
            //                     wall_state = WallStatus::InHorizontalWall;
            //                 },
            //                 // otherwise we're at a new wall, we were inside, so this is 

            //             }
            //         },
            //         WallStatus::InHorizontalWall => todo!(),
            //         WallStatus::InsideWall(c) => todo!(),
            //         WallStatus::JustExitedWall => todo!(),
            //     }
            // }
            // for x in self.bounds.0.x ..= self.bounds.1.x {
            //     let coord = (x, y).into();
            //     if self.walls.contains_key(&coord) {
            //         if !self.walls.contains_key(&coord.next(Direction::Left))
            //             && !self.walls.contains_key(&coord.next(Direction::Right))
            //             {
            //                 // we just crossed a wall going inwards
            //                 potential_interior.insert(coord.next(Direction::Right));
            //                 break 'a ;
            //             }
            //     } 
                // else {
                //     if !inside && wall {
                //     inside = true;
                //     potential_interior.insert(coord);
                //     break;
                //     }
                // }

        //         last_coord = Some(coord);
        //     }
        }

        // while let Some(next) = potential_interior.pop_first() {
        //     if self.walls.contains_key(&next) {
        //         continue;
        //     }

        //     if next.x < self.bounds.0.x || next.x > self.bounds.1.x
        //         || next.y < self.bounds.0.y || next.y > self.bounds.1.y {
        //             // out of bounds
        //             continue;
        //         }

        //     // otherwise this is 'outside'; check its neighbours, and skip any 'in-wall' ones
        //     for c in next.neighbours() {
        //         if !self.walls.contains_key(&c) && !determined_interior.contains(&c) {
        //             potential_interior.insert(c);
        //         }
        //     }

        //     determined_interior.insert(next);
        // }

        // determined_interior.len()
        interior_size
    }

}

pub fn parse_hex_digit(input: &str) -> IResult<&str, u8> {
    let (rest, (first, second)) = tuple((map(cc::anychar, |c| c.to_digit(16).unwrap()), map(cc::anychar, |c| c.to_digit(16).unwrap())))(input)?;
    Ok((rest, (first * 16 + second).try_into().unwrap()))
}

pub fn parse_instruction(input: &str) -> IResult<&str, (Direction, u64, (u8, u8, u8))> {
    let (rest, 
        (
        direction,
        _,
        number,
        _,
        _,
        (r, g, b),
        _)) =
        tuple((
        map(cc::anychar, Direction::from),
        cc::space1,
        cc::u64,
        cc::space1,
        tag("(#"),
        tuple((parse_hex_digit, parse_hex_digit, parse_hex_digit)),
        tag(")")
        ))(input)?;

    Ok((rest, (direction, number, (r, g, b))))
}

pub fn parse_input(input: &str) -> DigPlan {
    let mut instructions = Vec::new();
    for line in input.lines() {
        match all_consuming(parse_instruction)(line).finish() {
            Ok((_, instruction)) => instructions.push(instruction),
            Err(e) => { dbg!(e); unreachable!() }
        }
    }

    DigPlan {instructions}
}

pub fn parse_input_inverted(input: &str) -> DigPlan {
    let mut instructions = Vec::new();
    for line in input.lines() {
        match all_consuming(parse_instruction)(line).finish() {
            Ok((_, (_, _, (r, g, b)))) => {
                let digit1 = (r / 16) as u64;
                let digit2 = (r % 16) as u64;
                let digit3 = (g / 16) as u64;
                let digit4 = (g % 16) as u64;
                let digit5 = (b / 16) as u64;
                let direction = match b % 16 {
                    0 => Direction::Right,
                    1 => Direction::Down,
                    2 => Direction::Left,
                    3 => Direction::Up,
                    _ => unreachable!(),
                };
                let amount = digit5 + digit4 * 16 + digit3 * 16 * 16
                    + digit2 * 16 * 16 * 16 + digit1 * 16 * 16 * 16 * 16;
                instructions.push((direction, amount, (127, 127, 127)));
            }
            Err(e) => { dbg!(e); unreachable!() }
        }
    }

    DigPlan {instructions}
}


pub fn part_1(lagoon: &Lagoon) -> i64 {
    let size = (lagoon.bounds.1.y - lagoon.bounds.0.y + 1) * (lagoon.bounds.1.x - lagoon.bounds.0.x + 1);
    let exterior = lagoon.find_outside();
    let exterior_size = exterior.len() as i64;
    println!("Size is {}, exterior size is {}", size, exterior_size);
    size - exterior_size
}

pub fn part_1_interior(lagoon: &Lagoon) -> u64 {
    let size = (lagoon.bounds.1.y - lagoon.bounds.0.y + 1) * (lagoon.bounds.1.x - lagoon.bounds.0.x + 1);
    let interior = lagoon.find_inside();
    // let wall_size = lagoon.walls.keys().len();
    // println!("Size is {}, exterior size is {}", size, exterior_size);
    interior // + wall_size
}

fn main() {
    let input = include_str!("../input.txt");
    let digplan = parse_input(input);
    let lagoon = digplan.to_lagoon();
    // println!("{}", lagoon);
    // println!("Part 1: {}", part_1(&lagoon));
    println!("Part 1 (other method): {}", part_1_interior(&lagoon));

    let digplan2 = parse_input_inverted(input);
    // println!("Parsed");
    let lagoon2 = digplan2.to_lagoon();
    // println!("Made the lagoon");
    // println!("{}", lagoon);
    println!("Part 2: {}", part_1_interior(&lagoon2));
}

#[test]
pub fn test() {
    let input = r"R 6 (#70c710)
D 5 (#0dc571)
L 2 (#5713f0)
D 2 (#d2c081)
R 2 (#59c680)
D 2 (#411b91)
L 5 (#8ceee2)
U 2 (#caa173)
L 1 (#1b58a2)
U 2 (#caa171)
R 2 (#7807d2)
U 3 (#a77fa3)
L 2 (#015232)
U 2 (#7a21e3)";

    let digplan = parse_input(input);
    let lagoon = digplan.to_lagoon();
    // println!("{}", lagoon);
    // dbg!(&lagoon);
    // println!("{}", part_1(&lagoon));
    println!("{}", part_1_interior(&lagoon));

    
    let digplan2 = parse_input_inverted(input);
    // println!("Parsed");
    let lagoon2 = digplan2.to_lagoon();
    // println!("Made the lagoon");
    println!("Part 2: {}", part_1_interior(&lagoon2));
}