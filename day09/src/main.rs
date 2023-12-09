use itertools::Itertools;

pub fn predict_next_number(numbers: &[i64]) -> i64 {
    // base case
    if numbers.iter().all(|n| n == &0) {
        return 0;
    }

    let difference_sequence = numbers.iter()
        .tuple_windows()
        .map(|(first, second)| second - first)
        .collect::<Vec<_>>();
    let next_number = predict_next_number(&difference_sequence);
    *numbers.last().unwrap() + next_number
}

pub fn predict_previous_number(numbers: &[i64]) -> i64 {
    // base case
    if numbers.iter().all(|n| n == &0) {
        return 0;
    }

    let difference_sequence = numbers.iter()
        .tuple_windows()
        .map(|(first, second)| second - first)
        .collect::<Vec<_>>();
    let previous_number = predict_previous_number(&difference_sequence);
    *numbers.first().unwrap() - previous_number
}

pub fn parse_input(input: &str) -> Vec<Vec<i64>> {
    input.lines()
        .map(|line| line.split_whitespace()
            .map(|word| word.parse::<i64>().unwrap())
            .collect())
        .collect()
}

pub fn part_1(sequences: &[Vec<i64>]) -> i64 {
    sequences.iter()
        .map(|seq| predict_next_number(&seq))
        .sum()
}

pub fn part_2(sequences: &[Vec<i64>]) -> i64 {
    sequences.iter()
        .map(|seq| predict_previous_number(&seq))
        .sum()
}

fn main() {
    let input = include_str!("../input.txt");
    let sequences = parse_input(input);
    println!("Part 1: {}", part_1(&sequences));
    println!("Part 2: {}", part_2(&sequences));
}

#[test]
pub fn test() {
    let input = r"0 3 6 9 12 15
1 3 6 10 15 21
10 13 16 21 30 45";

    let sequences = parse_input(input);
    assert_eq!(predict_next_number(&sequences[0]), 18);
    assert_eq!(part_1(&sequences), 114);
    assert_eq!(part_2(&sequences), 2);
}