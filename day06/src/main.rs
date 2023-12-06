#[derive(Copy, Clone, Debug)]
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

pub fn solve(races: &[Race]) -> usize {
    races.iter()
        .map(|r|
            (0..=r.time)
                // formula for the distance: if you hold the button for h seconds, you go at h speed for (r.time-h) time
                .filter_map(|h| (h * (r.time - h) > r.distance).then_some(()))
                .count())
        .product()
}

pub fn solve_quadratic(races: &[Race]) -> u64 {
    races.iter()
        .map(|r| {
            // quadratic equation time
            let lower_root = (r.time as f64 - ((r.time * r.time - 4 * r.distance) as f64).sqrt()) / 2.0;
            // need to be careful: if the lower root is already an int, we need to skip it - so floor and add 1
            let lowest_value_working = lower_root.floor() as u64 + 1;
            // there are then that many values below that don't work (counting 0), and the same at the top by symmetry
            (r.time + 1).saturating_sub(lowest_value_working * 2)
        })
        .product()
}

pub fn main() {
    let input = include_str!("../input.txt");
    let races = parse_input(input);
    let single_race = parse_input_single(input);
    println!("Part 1: {}", solve(&races));
    println!("Part 2: {}", solve(&[single_race])); // 1.4s, 33ms in release
    println!("Part 2 (quad): {}", solve_quadratic(&[single_race])); // about 500us, 200us in release
}

#[test]
pub fn test() {
    let input = r"Time:      7  15   30
Distance:  9  40  200";

    let races = parse_input(input);
    assert_eq!(solve(&races), 288);
    let single_race = parse_input_single(input);
    assert_eq!(solve(&[single_race]), 71503);
    assert_eq!(solve_quadratic(&races), 288);
}