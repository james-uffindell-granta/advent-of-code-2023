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
use std::time::Instant;

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
        let mut vertices = Vec::new();
        let mut vertical_segments = Vec::new();
        let mut horizontal_segments = Vec::new();
        vertices.push(location);
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

            location = end_location;
            vertices.push(location);
        }

        let min_x = vertices.iter().map(|c| c.x).min().unwrap();
        let min_y = vertices.iter().map(|c| c.y).min().unwrap();
        let max_x = vertices.iter().map(|c| c.x).max().unwrap();
        let max_y= vertices.iter().map(|c| c.y).max().unwrap();

        let bounds = ((min_x, min_y).into(), (max_x, max_y).into());

        Lagoon { vertices, vertical_segments, horizontal_segments, bounds }
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
    vertices: Vec<Coord>,
    vertical_segments: Vec<VerticalSegment>,
    bounds: (Coord, Coord),
    horizontal_segments: Vec<HorizontalSegment>,
}

impl Lagoon {
    pub fn find_inside(&self) -> u64 {
        let mut interior_size = 0u64;

        // could make this faster by skipping any y that doesn't have vertices in
        // but we'd need to be more careful about tracking what's "open this row" - we don't want to drag a whole
        // block of horizontal row up
        for y in self.bounds.0.y ..= self.bounds.1.y {
            let mut vertical_walls_intersecting = self.vertical_segments.iter().filter_map(|s| s.intercept(y)).collect::<BTreeSet<_>>();
            let vertices_in_row = self.vertices.iter().filter(|c| c.y == y).map(|c| c.x).collect::<HashSet<_>>();
            let vertical_segment_lower_vertices = self.vertical_segments.iter()
                .filter(|s| s.lower.y == y)
                .map(|s| s.lower.x).collect::<HashSet<_>>();

            interior_size += vertical_walls_intersecting.len() as u64;

            let mut inside = false;
            let mut last_wall = None;
            while let Some(intercept) = vertical_walls_intersecting.pop_first() {
                if let Some(wall) = last_wall {
                    if inside {
                        interior_size += (intercept - wall - 1) as u64;
                    }
                }

                if vertices_in_row.contains(&intercept) {
                    // this must be the start of a horizontal line
                    // so the next intercept must be its end
                    let next = vertical_walls_intersecting.pop_first().unwrap();
                    assert!(vertices_in_row.contains(&next));

                    // include this many cells
                    // we include this regardless of whether we were previously "inside" or "outside"
                    interior_size += (next - intercept - 1) as u64;

                    match (vertical_segment_lower_vertices.contains(&intercept), vertical_segment_lower_vertices.contains(&next)) {
                        // both vertices are the bottom or the top of a segment -
                        // so this doesn't change our inside/outside parity
                        (true, true) | (false, false) => { },
                        // otherwise this was a horizontal jog in a vertical line
                        // so we have changed whether we're inside or outside
                        (true, false) | (false, true) => { inside = !inside; },
                    }

                    last_wall = Some(next);
                } else {
                    // this is a vertical wall going past us
                    inside = !inside;
                    last_wall = Some(intercept);
                }
            }
        }

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

pub fn part_1_interior(lagoon: &Lagoon) -> u64 {
    lagoon.find_inside()
}

pub fn part_1_pick(lagoon: &Lagoon) -> u64 {
    let area: i64 = lagoon.vertices.iter().tuple_windows().map(|(c1, c2)| ((c1.y + c2.y) * (c1.x - c2.x))).sum::<i64>() / 2i64;
    let perimeter: u64 = lagoon.horizontal_segments.iter().map(|s| (s.upper.x - s.lower.x) as u64).sum::<u64>()
        + lagoon.vertical_segments.iter().map(|s| (s.upper.y - s.lower.y) as u64).sum::<u64>();
    area as u64 + (perimeter / 2) + 1
}

fn main() {
    let input = include_str!("../input.txt");
    let digplan = parse_input(input);
    let lagoon = digplan.to_lagoon();
    println!("Part 1: {}", part_1_interior(&lagoon));

    let digplan2 = parse_input_inverted(input);
    let lagoon2 = digplan2.to_lagoon();
    let now = Instant::now();
    println!("Part 2: {}", part_1_pick(&lagoon2));
    println!("Part 2 took: {:2?}", now.elapsed());
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
    println!("{}", part_1_interior(&lagoon));
    println!("{}", part_1_pick(&lagoon));

    let digplan2 = parse_input_inverted(input);
    let lagoon2 = digplan2.to_lagoon();
    println!("{}", part_1_pick(&lagoon2));
}