use std::collections::{HashMap, HashSet};

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug)]
pub struct Coord {
    x: i64,
    y: i64,
}

impl Coord {
    pub fn points_around_line_to(&self, other: &Coord) -> HashSet<Coord> {
        assert!(self.y == other.y);
        assert!(self.x <= other.x);
        
        let mut coords = HashSet::new();
        coords.insert((self.x - 1, self.y).into());
        coords.insert((other.x + 1, self.y).into());
        for x in (self.x - 1)..=(other.x + 1) {
            coords.insert((x, self.y + 1).into());
            coords.insert((x, self.y - 1).into());
        }

        coords
    }
}

impl From<(i64, i64)> for Coord {
    fn from(value: (i64, i64)) -> Self {
        Coord { x: value.0, y: value.1 }
    }
}

#[derive(Debug)]
pub struct Schematic {
    // seems like we don't need to remember which symbol is where yet
    symbol_locations: HashSet<Coord>,
    // other than gears for part 2
    gear_locations: HashSet<Coord>,
    // we need to remember numbers and their start locations
    // numbers might repeat so key on the start location instead
    // we need to easily figure out the length so just save the unparsed number for now
    number_locations: HashMap<Coord, String>,
}

pub fn parse_input(input: &str) -> Schematic {
    let mut symbol_locations = HashSet::new();
    let mut gear_locations = HashSet::new();
    let mut number_locations = HashMap::new();
    for (y, line) in input.lines().enumerate() {
        let mut current_number = String::new();
        let mut start_location = None;
        for (x, c) in line.chars().enumerate() {
            if c.is_ascii_digit() {
                current_number.push(c);
                if start_location.is_none() {
                    // first digit of a new number
                    start_location = Some((x as i64, y as i64).into());
                }
            } else {
                if let Some(location) = start_location {
                    // we were parsing a number and now we've reached a non-digit
                    number_locations.insert(location, current_number);
                    start_location = None;
                    current_number = String::new();
                }

                if c != '.' {
                    // found a symbol
                    symbol_locations.insert((x as i64, y as i64).into());
                    if c == '*' {
                        gear_locations.insert((x as i64, y as i64).into());
                    }
                }
            }
        }

        if let Some(location) = start_location {
            // we were parsing a number and now we've reached the end of the line
            number_locations.insert(location, current_number);
        }
    }

    Schematic { symbol_locations, gear_locations, number_locations }
}

pub fn part_1(schematic: &Schematic) -> u64 {
    let mut total = 0;
    for (start_coord, number) in &schematic.number_locations {
        let end_coord = Coord { x: start_coord.x + number.len() as i64 - 1, ..*start_coord };
        let surrounding_points = start_coord.points_around_line_to(&end_coord);
        if let Some(_symbol) = surrounding_points.intersection(&schematic.symbol_locations).next() {
            total += number.parse::<u64>().unwrap();
        }
    }

    total
}

pub fn part_2(schematic: &Schematic) -> u64 {
    let mut gear_numbers = HashMap::new();
    for (start_coord, number) in &schematic.number_locations {
        let end_coord = Coord { x: start_coord.x + number.len() as i64 - 1, ..*start_coord };
        let surrounding_points = start_coord.points_around_line_to(&end_coord);
        for gear in surrounding_points.intersection(&schematic.gear_locations) {
            // remember that this number is next to this gear
            gear_numbers.entry(*gear).or_insert(Vec::new()).push(number);
        }
    }

    gear_numbers.values()
        .filter(|ns| ns.len() == 2)
        .map(|ns| ns.into_iter()
            .map(|n| n.parse::<u64>().unwrap())
            .product::<u64>())
        .sum()

}

fn main() {
    let input = include_str!("../input.txt");
    let schematic = parse_input(input);
    println!("Part 1: {}", part_1(&schematic));
    println!("Part 2: {}", part_2(&schematic));
}

#[test]
pub fn test() {
    let input = r"467..114..
...*......
..35..633.
......#...
617*......
.....+.58.
..592.....
......755.
...$.*....
.664.598..";

    let schematic = parse_input(input);
    assert_eq!(part_1(&schematic), 4361);
    assert_eq!(part_2(&schematic), 467835);
}
