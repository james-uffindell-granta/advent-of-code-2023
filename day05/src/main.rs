use std::{collections::HashMap,
    hash::Hash,
ops::Range};
use itertools::Itertools;
use interval::{IntervalSet, ops::Range as _, interval_set::ToIntervalSet};
use gcollections::ops::{Difference, Join, set::Intersection, Bounded, Union};

// started off with u64 but i safer
#[derive(PartialEq, Eq, Hash, Copy, Clone, Debug)]
pub struct Mapping {
    source_start: i64,
    destination_start: i64,
    length: i64,
}

impl Mapping {
    pub fn get_mapped_value(&self, input: i64) -> Option<i64> {
        if (self.source_start .. self.source_start + self.length).contains(&input) {
            Some(self.destination_start + (input - self.source_start))
        } else {
            None
        }
    }

    // returns (new values, mapped portion)
    pub fn get_mapped_segment(&self, input: IntervalSet<i64>) -> (IntervalSet<i64>, IntervalSet<i64>) {
        let source_interval = vec![(self.source_start, self.source_start + self.length - 1)].to_interval_set();
        // let input_interval = vec![(input.start, input.end - 1)].to_interval_set();
        // everything in the intersection is mapped
        let intersection = source_interval.intersection(&input);
        // the way things are mapped: we add d - s to the everything.
        let mapped_intersection = intersection.clone() + (self.destination_start - self.source_start);
        (mapped_intersection, intersection)
    }

    pub fn get_mapped_range(&self, input: IntervalSet<i64>) -> (IntervalSet<i64>, IntervalSet<i64>) {
        // the mapping says that [s, s + l) maps to [d, d + l)
        // we have [a, a + x).

        // case 1: input entirely outside this map; nothing to do, return input.
        // if input.end <= self.source_start || (self.source_start + self.length) <= input.start {
        //     return (0..0, input);
        // }
        
        // // case 2: input entirely within this map; map everything
        // if input.start >= self.source_start || input.end <= self.source_start + self.length {
        //     let input_length = input.end - input.start;
        //     let mapped_start = self.destination_start + (self.source_start - input.start);
        //     return (mapped_start..(mapped_start + input_length), 0..0);
        // }

        // wait don't do this

        let source_interval = vec![(self.source_start, self.source_start + self.length - 1)].to_interval_set();
        // let input_interval = vec![(input.start, input.end - 1)].to_interval_set();
        // everything in the intersection is mapped
        let intersection = source_interval.intersection(&input);
        // the way things are mapped: we add d - s to the everything.
        let mapped_intersection = intersection.clone() + (self.destination_start - self.source_start);

        // everything in the rest is left alone
        let rest = input.difference(&intersection);

        (mapped_intersection, rest)
    }
}

#[derive(Clone, PartialEq, Eq, Hash)]
pub struct RangeSet {
    ranges: Vec<Range<i64>>,
}

impl RangeSet {
    pub fn extend(&mut self, other: &RangeSet) {
        self.ranges.extend(other.ranges.clone());
    }

    pub fn transform(&self, mapping: &Mapping) -> (RangeSet, RangeSet) {
        let mut transformed_ranges = Vec::new();
        let mut unmapped_ranges = Vec::new();
        let source_end = mapping.source_start + mapping.length;
        let adjustment = mapping.destination_start - mapping.source_start;
        for r in &self.ranges {
            if r.end <= mapping.source_start || r.start >= source_end {
                unmapped_ranges.push(r.clone());
            } else if r.start < mapping.source_start && r.end <= source_end {
                // r.end must be in the middle
                unmapped_ranges.push(r.start .. mapping.source_start);
                transformed_ranges.push(mapping.source_start + adjustment .. r.end + adjustment);
            } else if r.start < mapping.source_start && r.end > source_end {
                unmapped_ranges.push(r.start .. mapping.source_start);
                unmapped_ranges.push(source_end .. r.end);
                transformed_ranges.push(mapping.source_start + adjustment .. source_end + adjustment);
            } else if r.end <= source_end {
                // whole range is mapped
                transformed_ranges.push(r.start + adjustment .. r.end + adjustment);
            } else {
                // start between, end outside
                unmapped_ranges.push(source_end .. r.end);
                transformed_ranges.push(r.start + adjustment .. source_end + adjustment);
            }
        }

        (RangeSet { ranges: transformed_ranges }, RangeSet { ranges: unmapped_ranges })
    }

    pub fn transform_all(&self, map: &FullMap) -> RangeSet {
        let mut transformed_ranges = RangeSet { ranges: vec![] };
        let mut unmapped_ranges = self.clone();

        for m in &map.mappings {
            let (transformed, unmapped) = unmapped_ranges.transform(m);
            transformed_ranges.extend(&transformed);
            unmapped_ranges = unmapped;
        }

        // everything else is mapped straight through
        transformed_ranges.extend(&unmapped_ranges);

        transformed_ranges
    }
}

#[derive(Debug)]
pub struct FullMap {
    mappings: Vec<Mapping>,
}

impl FullMap {
    pub fn get_mapped_value(&self, input: i64) -> i64 {
        for m in &self.mappings {
            if let Some(result) = m.get_mapped_value(input) {
                return result;
            }
        }

        input
    }
    pub fn get_mapped_ranges(&self, range: IntervalSet<i64>) -> IntervalSet<i64> {
        let mut final_values = vec![].to_interval_set();
        let mut mapped_portions = vec![].to_interval_set();
        for m in &self.mappings {
            let (new_values, mapped_portion) = m.get_mapped_segment(range.clone());
            final_values = final_values.union(&new_values);
            mapped_portions = mapped_portions.union(&mapped_portion);
        }

        let unmapped_portion = range.difference(&mapped_portions);

        final_values.union(&unmapped_portion)
    }
}

pub fn build_map(input: &str) -> FullMap {
    let mut mappings = Vec::new();
    let mut lines = input.lines();
    _ = lines.next(); // ignore first line with text on
    for line in lines {
        let (destination_start, source_start, length) = line.split_whitespace().map(|s| s.parse::<i64>().unwrap()).collect_tuple().unwrap();
        mappings.push(Mapping { source_start, destination_start, length });
    }

    FullMap { mappings }
}

#[derive(Debug)]
pub struct SeedRange {
    start: i64,
    length: i64,
}

#[derive(Debug)]
pub struct Input {
    seeds: Vec<i64>,
    seed_ranges: Vec<SeedRange>,
    seed_to_soil: FullMap,
    soil_to_fertilizer: FullMap,
    fertilizer_to_water: FullMap,
    water_to_light: FullMap,
    light_to_temperature: FullMap,
    temperature_to_humidity: FullMap,
    humidity_to_location: FullMap,
}

impl Input {
    pub fn seed_locations(&self) -> Vec<i64> {
        self.locations_for(&self.seeds)
    }

    pub fn locations_for(&self, seeds: &[i64]) -> Vec<i64> {
        seeds.iter()
            .map(|seed| self.seed_to_soil.get_mapped_value(*seed))
            .map(|soil| self.soil_to_fertilizer.get_mapped_value(soil))
            .map(|fertilizer| self.fertilizer_to_water.get_mapped_value(fertilizer))
            .map(|water| self.water_to_light.get_mapped_value(water))
            .map(|light| self.light_to_temperature.get_mapped_value(light))
            .map(|temperature| self.temperature_to_humidity.get_mapped_value(temperature))
            .map(|humidity| self.humidity_to_location.get_mapped_value(humidity))
            .collect()
    }

    pub fn best_seed_range_locations(&self) -> i64 {
        let mut seeds = self.seed_ranges.iter().map(|s| (s.start, s.start + s.length - 1)).collect::<Vec<_>>();
        seeds.sort_by_key(|(s, _)| *s);
        let seed_set = seeds.to_interval_set();
        let soil_set = self.seed_to_soil.get_mapped_ranges(seed_set);
        let fertilizer_set = self.soil_to_fertilizer.get_mapped_ranges(soil_set);
        let water_set = self.fertilizer_to_water.get_mapped_ranges(fertilizer_set);
        let light_set = self.water_to_light.get_mapped_ranges(water_set);
        let temperature_set = self.light_to_temperature.get_mapped_ranges(light_set);
        let humidity_set = self.temperature_to_humidity.get_mapped_ranges(temperature_set);
        let location_set = self.humidity_to_location.get_mapped_ranges(humidity_set);
        location_set.lower()
        // soil_set.lower()
    }

    pub fn best_seed_range_locations_local(&self) -> i64 {
        let seeds = RangeSet { ranges: self.seed_ranges.iter().map(|s| s.start .. s.start + s.length).collect::<Vec<_>>() };
        let mut result = seeds.transform_all(&self.seed_to_soil);
        result = result.transform_all(&self.soil_to_fertilizer);
        result = result.transform_all(&self.fertilizer_to_water);
        result = result.transform_all(&self.water_to_light);
        result = result.transform_all(&self.light_to_temperature);
        result = result.transform_all(&self.temperature_to_humidity);
        result = result.transform_all(&self.humidity_to_location);
        result.ranges.iter().map(|r| r.start).min().unwrap()
    }
}

pub fn parse_input(input: &str) -> Input {
    let mut chunks = input.split("\n\n");

    let seeds: Vec<_> = chunks.next().unwrap().split_whitespace().skip(1).map(|s| s.parse::<i64>().unwrap()).collect();
    let seed_ranges = seeds.clone().into_iter().tuples().map(|(s, l)| SeedRange { start: s, length: l }).collect();
    let seed_to_soil = build_map(chunks.next().unwrap());
    let soil_to_fertilizer = build_map(chunks.next().unwrap());
    let fertilizer_to_water = build_map(chunks.next().unwrap());
    let water_to_light = build_map(chunks.next().unwrap());
    let light_to_temperature = build_map(chunks.next().unwrap());
    let temperature_to_humidity = build_map(chunks.next().unwrap());
    let humidity_to_location = build_map(chunks.next().unwrap());

    Input {
        seeds,
        seed_ranges,
        seed_to_soil,
        soil_to_fertilizer,
        fertilizer_to_water,
        water_to_light,
        light_to_temperature,
        temperature_to_humidity,
        humidity_to_location,
    }
}

pub fn part_1(input: &Input) -> i64 {
    *input.seed_locations().iter().min().unwrap()
}

pub fn part_2(input: &Input) -> i64 {
    input.best_seed_range_locations()
}

pub fn part_2b(input: &Input) -> i64 {
    input.best_seed_range_locations_local()
}

fn main() {
    let input = include_str!("../input.txt");
    let input = parse_input(input);
    println!("Part 1: {}", part_1(&input));
    println!("Part 2: {}", part_2(&input));
    println!("Part 2b: {}", part_2b(&input));
}

#[test]
pub fn test() {
    let input = r"seeds: 79 14 55 13

seed-to-soil map:
50 98 2
52 50 48

soil-to-fertilizer map:
0 15 37
37 52 2
39 0 15

fertilizer-to-water map:
49 53 8
0 11 42
42 0 7
57 7 4

water-to-light map:
88 18 7
18 25 70

light-to-temperature map:
45 77 23
81 45 19
68 64 13

temperature-to-humidity map:
0 69 1
1 0 69

humidity-to-location map:
60 56 37
56 93 4";

    let input = parse_input(input);
    assert_eq!(part_1(&input), 35);
    assert_eq!(part_2(&input), 46);
    assert_eq!(part_2b(&input), 46);
}

 