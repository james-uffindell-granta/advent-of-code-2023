use aho_corasick::AhoCorasick;

pub fn line_to_number(input: &str) -> u32 {
    let digits = input.chars()
        .filter(|c| c.is_ascii_digit()).collect::<Vec<_>>();

    digits.first().unwrap().to_digit(10).unwrap() * 10 + digits.last().unwrap().to_digit(10).unwrap()
}

pub fn line_to_word_number(input: &str) -> u32 {
    // in order so pattern n maps to number (n / 2 + 1)
    let numbers = AhoCorasick::new([
        "one", "1", "two", "2", "three", "3", "four", "4", "five", "5", "six", "6", "seven", "7", "eight", "8", "nine", "9"
    ]).unwrap();

    let matches = numbers.find_overlapping_iter(input).map(|m| m.pattern().as_u32() / 2 + 1).collect::<Vec<_>>();
    matches.first().unwrap() * 10 + matches.last().unwrap()
}

pub fn part_1(input: &str) -> u32 {
    input.lines().map(line_to_number).sum()
}

pub fn part_2(input: &str) -> u32 {
    input.lines().map(line_to_word_number).sum()
}

fn main() {
    let input = include_str!("../input.txt");   
    println!("Part 1: {}", part_1(input)); 
    println!("Part 2: {}", part_2(input));
}

#[test]
pub fn test() {
    assert_eq!(line_to_word_number("two1nine"), 29);
    assert_eq!(line_to_word_number("eightwothree"), 83);
    assert_eq!(line_to_word_number("abcone2threexyz"), 13);
    assert_eq!(line_to_word_number("xtwone3four"), 24);
    assert_eq!(line_to_word_number("4nineeightseven2"), 42);
    assert_eq!(line_to_word_number("zoneight234"), 14);
    assert_eq!(line_to_word_number("7pqrstsixteen"), 76);
    assert_eq!(line_to_word_number("7fiveeightoneightvs"), 78);
    assert_eq!(line_to_word_number("eightwo"), 82);
}
