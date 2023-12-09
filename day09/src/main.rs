use itertools::Itertools;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum Direction { Forwards, Backwards }

pub fn predict_additional_number(numbers: &[i64], direction: Direction) -> i64 {
    // base case - always 0 either way we go
    if numbers.iter().all(|n| n == &0) {
        return 0;
    }

    let difference_sequence = numbers.iter()
        .tuple_windows()
        .map(|(first, second)| second - first)
        .collect::<Vec<_>>();
    let additional_number = predict_additional_number(&difference_sequence, direction);
    match direction {
        Direction::Forwards => *numbers.last().unwrap() + additional_number,
        Direction::Backwards => *numbers.first().unwrap() - additional_number,
    }
}

pub fn parse_input(input: &str) -> Vec<Vec<i64>> {
    input.lines()
        .map(|line| line.split_whitespace()
            .map(|word| word.parse::<i64>().unwrap())
            .collect())
        .collect()
}

pub fn solve(sequences: &[Vec<i64>], direction: Direction) -> i64 {
    sequences.iter()
        .map(|seq| predict_additional_number(&seq, direction))
        .sum()
}

fn main() {
    let input = include_str!("../input.txt");
    let sequences = parse_input(input);
    println!("Part 1: {}", solve(&sequences, Direction::Forwards));
    println!("Part 2: {}", solve(&sequences, Direction::Backwards));
}

#[test]
pub fn test() {
    let input = r"0 3 6 9 12 15
1 3 6 10 15 21
10 13 16 21 30 45";

    let sequences = parse_input(input);
    assert_eq!(solve(&sequences, Direction::Forwards), 114);
    assert_eq!(solve(&sequences, Direction::Backwards), 2);
}