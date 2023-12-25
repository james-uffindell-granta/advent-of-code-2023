use std::{ops::{Add, Mul}, collections::HashMap};
use itertools::Itertools;
use std::time::Instant;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Coord2 { x: i128, y: i128 }


#[derive(Copy, Clone, PartialEq, Debug)]
pub struct Coord2F { x: f64, y: f64 }

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Coord3 { x: i128, y: i128, z: i128 }

impl From<(i128, i128)> for Coord2 {
    fn from((x, y): (i128, i128)) -> Self {
        Self { x, y }
    }
}

impl Add<(i128, i128)> for Coord2 {
    type Output = Coord2;

    fn add(self, (x, y): (i128, i128)) -> Self::Output {
        (self.x + x, self.y + y).into()
    }
}

impl From<(i128, i128, i128)> for Coord3 {
    fn from((x, y, z): (i128, i128, i128)) -> Self {
        Self { x, y, z }
    }
}

impl Add<(i128, i128, i128)> for Coord3 {
    type Output = Coord3;

    fn add(self, (x, y, z): (i128, i128, i128)) -> Self::Output {
        (self.x + x, self.y + y, self.z + z).into()
    }
}

impl Add<Coord3> for Coord3 {
    type Output = Coord3;

    fn add(self, rhs: Coord3) -> Self::Output {
        self + (rhs.x, rhs.y, rhs.z)
    }
}

impl Mul<i128> for Coord3 {
    type Output = Coord3;

    fn mul(self, rhs: i128) -> Self::Output {
        (self.x * rhs, self.y * rhs, self.z * rhs).into()
    }
}

impl Coord3 {
    pub fn ignoring_z(&self) -> Coord2 {
        Coord2 { x: self.x, y: self.y }
    }

    pub fn dot(&self, other: Coord3) -> i128 {
        self.x * other.x + self.y * other.y + self.z * other.z
    }

    pub fn cross(&self, other: Coord3) -> Coord3 {
        (self.y * other.z - self.z * other.y,
        self.z * other.x - self.x * other.z,
        self.x * other.y - self.y * other.x).into()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Position2(Coord2);

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Velocity2(Coord2);

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Line2 {
    pos: Position2,
    v: Velocity2,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum IntersectionType<T> {
    NoIntersection,
    HistoricalIntersection(T),
    FutureIntersection(T),
    FullyOverlapping,
}

impl Line2 {
    pub fn intersection_with(&self, other: &Self) -> IntersectionType<Coord2F> {
        // trust me on this maths
        // self.pos.x + a*self.v.x = other.pos.x + b*other.v.x
        // self.pos.y + a*self.v.y = other.pos.y + b*other.v.y
        // / self.v.x  -other.v.x \/ a \ = / other.pos.x - self.pos.x \
        // \ self.v.y  -other.v.y /\ b / = \ other.pos.y - self.pos.y /
        // so find the determinant
        let det = (other.v.0.x * self.v.0.y) - (self.v.0.x * other.v.0.y);
        if det == 0 {
            // not invertible - two options
            // either the lines are identical, or they are non-intersecting
            if other.v.0.x * self.v.0.y == other.v.0.y * self.v.0.x
                && self.v.0.x * (other.pos.0.y - self.pos.0.y) == self.v.0.y * (other.pos.0.x - self.pos.0.x) {
                    return IntersectionType::FullyOverlapping;
                } 

            return IntersectionType::NoIntersection;
        }

        let a_unnormalized = (other.v.0.x * (other.pos.0.y - self.pos.0.y)) - (other.v.0.y * (other.pos.0.x - self.pos.0.x));
        let b_unnormalized = (self.v.0.x * (other.pos.0.y - self.pos.0.y)) - (self.v.0.y * (other.pos.0.x - self.pos.0.x));
        let a = a_unnormalized as f64 / det as f64;
        // don't need to figure out b, the intersection will be the same for both lines
        let b = b_unnormalized as f64 / det as f64;
        let intersection_point = Coord2F { x: self.pos.0.x as f64 + a * self.v.0.x as f64, y: self.pos.0.y as f64 + a * self.v.0.y as f64 };
        if a < 0.0 || b < 0.0 {
            IntersectionType::HistoricalIntersection(intersection_point)
        } else {
            IntersectionType::FutureIntersection(intersection_point)
        }

    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Position3(Coord3);

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Velocity3(Coord3);

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub struct Line3 {
    pos: Position3,
    v: Velocity3,
}

impl Line3 {
    pub fn ignoring_z(&self) -> Line2 {
        Line2 { pos: Position2(self.pos.0.ignoring_z()), v: Velocity2(self.v.0.ignoring_z()) }
    }
}

pub fn parse_input(input: &str) -> Vec<Line3> {
    let mut lines = Vec::new();
    for line in input.lines() {
        let (position, velocity) = line.split_once(" @ ").unwrap();
        let ps @ (_, _, _) = position.split(',').map(|s| s.trim().parse::<i128>().unwrap()).collect_tuple().unwrap();
        let vs @ (_, _, _) = velocity.split(',').map(|s| s.trim().parse::<i128>().unwrap()).collect_tuple().unwrap();
        lines.push(Line3 { pos: Position3(ps.into()), v: Velocity3(vs.into()) })
    }

    lines
}

pub fn part_1(lines: &[Line3]) -> u64 {
    let lines_ignoring_z = lines.iter().map(|l| l.ignoring_z()).collect::<Vec<_>>();
    let mut count = 0;
    let lower_bound = 200_000_000_000_000.0_f64;
    let upper_bound = 400_000_000_000_000.0_f64;
    // let lower_bound = 7.0_f64;
    // let upper_bound = 27.0_f64;

    for (line1, line2) in lines_ignoring_z.iter().tuple_combinations() {
        match line1.intersection_with(line2) {
            IntersectionType::FutureIntersection(c) => {
                if c.x >= lower_bound && c.y >= lower_bound
                    && c.x <= upper_bound && c.y <= upper_bound {
                        count += 1;
                    }
            },
            _ => { },
        }
    }

    count
}

pub fn intersection_2d_with_offset(lines: &[Line3], velocity_offset: Coord2) -> IntersectionType<Coord2> {
    let mut intersection = None;
    let adjusted_lines = lines.iter().map(|l| l.ignoring_z())
        .map(|l| Line2 {
            pos: l.pos,
            v: Velocity2((l.v.0.x - velocity_offset.x, l.v.0.y - velocity_offset.y).into())
        }).collect::<Vec<_>>();
    // if velocity_offset == (-3, 1).into() {
    //     println!("{:?}", adjusted_lines);
    // }

    for (line1, line2) in adjusted_lines.iter().tuple_combinations() {
        // if velocity_offset == (-227, -221).into() {
        //     println!("Intersecting {:?} with {:?}, intersection type is {:?}", line1, line2, line1.intersection_with(line2));
        // }
        match line1.intersection_with(line2) {
            IntersectionType::FutureIntersection(c) => {
                let int_c = Coord2 { x: c.x.round() as u64 as i128, y: c.y.round() as u64 as i128 };
                match intersection {
                    None => intersection = Some(IntersectionType::FutureIntersection(int_c)),
                    Some(IntersectionType::FutureIntersection(i)) => {
                        if int_c == i {
                            continue;
                        } else {
                            return IntersectionType::NoIntersection;
                        }
                    },
                    _ => unreachable!(),
                }
            },
            IntersectionType::FullyOverlapping => {
                // not really sure how best to check if this is consistent - maybe just skip and hope?
                continue
            }
            _ => return IntersectionType::NoIntersection,
        }
    }

    intersection.unwrap()
}

pub fn part_2(input: &[Line3]) -> i128 {
    // 1025019997186820
    // translate to the rock's frame of reference: 
    // this amounts to finding the common intersection point of all the hailstones
    // (they all hit each other at once, where the rock 'sits')

    let now = Instant::now();
    let max_to_consider = 1_000;
    let mut x_y_intersections = HashMap::new();
    for v_x in (0 ..= max_to_consider).flat_map(|x| [x, -x]) {
        for v_y in (0 ..= max_to_consider).flat_map(|y| [y, -y]) {
            // if v_x == -227 && v_y == -221 {
            //     println!("Considering -3, 1");
            // }
            // consider all possible rock velocities in this range and see if the modified x-y lines intersect
            match intersection_2d_with_offset(input, (v_x, v_y).into()) {
                s @ IntersectionType::FutureIntersection(c) => { 
                    x_y_intersections.insert((v_x, v_y), c);
                },
                _ => continue,
            }
        }
    }

    dbg!(&x_y_intersections);

    let mut results = Vec::new();
    for ((x_offset, y_offset), potential_intersection) in x_y_intersections {
        // println!("First line is: {:?}", input[0]);
        let t1 = (potential_intersection.x as f64 - input[0].pos.0.x as f64) / (input[0].v.0.x - x_offset) as f64;
        let w1 = input[0].v.0.z;
        let z1 = input[0].pos.0.z;
        // println!("Intersection time for first line is: {}", t1);
        // let z_intercept = input[0].pos.0.z + (input[0].v.0.z as f64 * intersection_time) as i128;
        // println!("First hailstone at that time is at: {}, {}, {}", potential_intersection.x, potential_intersection.y, z_intercept);
        
        let t2 = (potential_intersection.x as f64 - input[1].pos.0.x as f64) / (input[1].v.0.x - x_offset) as f64;
        // println!("Intersection time for second line is: {}", t2);
        let w2 = input[1].v.0.z;
        let z2 = input[1].pos.0.z;
        // let z_intercept = input[0].pos.0.z + (input[0].v.0.z as f64 * intersection_time) as i128;
        let t1 = t1 as i128;
        let t2 = t2 as i128;

        let rock_z = (t2 * (z1 + t1 * w1) - t1 * (z2 + t2 * w2)) / (t2 - t1);
        let rock_w = (-z1 - t1 * w1 + z2 + t2 * w2) / (t2 - t1);

        // println!("Coords in question are: {}, {}, {}, {}", z1, w1, z2, w2);
        // println!("Rock start is at: {}, {}, {}, {}", potential_intersection.x, potential_intersection.y, rock_z, rock_w);
        results.push(potential_intersection.x + potential_intersection.y + rock_z);
    }
    println!("Took {:2?}", now.elapsed());

    if results.len() > 1 {
        panic!("Too many results, don't know what to do");
    }

    *results.first().unwrap()
}

fn main() {
    let input = include_str!("../input.txt");
    let lines = parse_input(input);
    println!("Part 1: {}", part_1(&lines));
    println!("Part 2: {}", part_2(&lines));
}

#[test]
pub fn test() {
    let input = r"19, 13, 30 @ -2,  1, -2
18, 19, 22 @ -1, -1, -2
20, 25, 34 @ -2, -2, -4
12, 31, 28 @ -1, -2, -1
20, 19, 15 @  1, -5, -3";

    let lines = parse_input(input);
    dbg!(part_1(&lines));
    dbg!(part_2(&lines));
}
