
#[derive(Debug)]
pub struct Race {
    time: u64,
    distance: u64,
}

pub fn parse_input(input: &str) -> Vec<Race> {
    let mut lines = input.lines();
    let times = lines.next().unwrap().split_whitespace().skip(1).map(|t| t.parse().unwrap());
    let distances = lines.next().unwrap().split_whitespace().skip(1).map(|d| d.parse().unwrap());
    times.zip(distances).map(|(time, distance)| Race { time, distance }).collect()
}

pub fn parse_input_single(input: &str) -> Race {
    let mut lines = input.lines();
    let time = lines.next().unwrap().split_whitespace().skip(1).collect::<String>().parse().unwrap();
    let distance = lines.next().unwrap().split_whitespace().skip(1).collect::<String>().parse().unwrap();
    Race { time, distance }
}

pub fn part_1(races: &[Race]) -> usize {
    races.iter()
        .map(|r| (0..=r.time).filter_map(|h| (h * (r.time - h) > r.distance).then_some(())).count())
        .product()
}

pub fn main() {
    let input = include_str!("../input.txt");
    let races = parse_input(input);
    let single_race = parse_input_single(input);
    println!("Part 1: {}", part_1(&races));
    println!("Part 2: {}", part_1(&[single_race]));
}

#[test]
pub fn test() {
    let input = r"Time:      7  15   30
Distance:  9  40  200";

    let races = dbg!(parse_input(input));
    assert_eq!(part_1(&races), 288);
    let single_race = parse_input_single(input);

    assert_eq!(part_1(&[single_race]), 71503);
}