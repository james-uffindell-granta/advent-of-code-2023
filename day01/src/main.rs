use std::collections::BTreeMap;
use regex::Regex;

pub fn line_to_number(input: &str) -> u32 {
    let digits = input.chars()
        .filter(|c| c.is_ascii_digit()).collect::<Vec<_>>();
    let number = format!("{}{}", digits.first().unwrap(), digits.last().unwrap());
    number.parse().unwrap()
}

pub fn line_to_word_number(input: &str) -> u32 {
    let mut first_indices = BTreeMap::new();
    let mut second_indices = BTreeMap::new();

    // bug here first time - just finding the first example!
    let ones = Regex::new("one|1").unwrap().find_iter(input).map(|m| m.start()).collect::<Vec<_>>();
    if let Some(first_one) = ones.first() {
        if let Some(last_one) = ones.last() {
            first_indices.insert(first_one, 1);
            second_indices.insert(last_one, 1);
        }
    }

    let ones = Regex::new("two|2").unwrap().find_iter(input).map(|m| m.start()).collect::<Vec<_>>();
    if let Some(first_one) = ones.first() {
        if let Some(last_one) = ones.last() {
            first_indices.insert(first_one, 2);
            second_indices.insert(last_one, 2);
        }
    }

    let ones = Regex::new("three|3").unwrap().find_iter(input).map(|m| m.start()).collect::<Vec<_>>();
    if let Some(first_one) = ones.first() {
        if let Some(last_one) = ones.last() {
            first_indices.insert(first_one, 3);
            second_indices.insert(last_one, 3);
        }
    }

    let ones = Regex::new("four|4").unwrap().find_iter(input).map(|m| m.start()).collect::<Vec<_>>();
    if let Some(first_one) = ones.first() {
        if let Some(last_one) = ones.last() {
            first_indices.insert(first_one, 4);
            second_indices.insert(last_one, 4);
        }
    }

    let ones = Regex::new("five|5").unwrap().find_iter(input).map(|m| m.start()).collect::<Vec<_>>();
    if let Some(first_one) = ones.first() {
        if let Some(last_one) = ones.last() {
            first_indices.insert(first_one, 5);
            second_indices.insert(last_one, 5);
        }
    }

    let ones = Regex::new("six|6").unwrap().find_iter(input).map(|m| m.start()).collect::<Vec<_>>();
    if let Some(first_one) = ones.first() {
        if let Some(last_one) = ones.last() {
            first_indices.insert(first_one, 6);
            second_indices.insert(last_one, 6);
        }
    }

    let ones = Regex::new("seven|7").unwrap().find_iter(input).map(|m| m.start()).collect::<Vec<_>>();
    if let Some(first_one) = ones.first() {
        if let Some(last_one) = ones.last() {
            first_indices.insert(first_one, 7);
            second_indices.insert(last_one, 7);
        }
    }

    let ones = Regex::new("eight|8").unwrap().find_iter(input).map(|m| m.start()).collect::<Vec<_>>();
    if let Some(first_one) = ones.first() {
        if let Some(last_one) = ones.last() {
            first_indices.insert(first_one, 8);
            second_indices.insert(last_one, 8);
        }
    }

    let ones = Regex::new("nine|9").unwrap().find_iter(input).map(|m| m.start()).collect::<Vec<_>>();
    if let Some(first_one) = ones.first() {
        if let Some(last_one) = ones.last() {
            first_indices.insert(first_one, 9);
            second_indices.insert(last_one, 9);
        }
    }

    let answer = format!("{}{}", first_indices.first_key_value().unwrap().1,second_indices.last_key_value().unwrap().1);
    answer.parse().unwrap()
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
    // this was the one in my input that messed me up
    assert_eq!(line_to_word_number("7fiveeightoneightvs"), 78);
    assert_eq!(line_to_word_number("eightwo"), 82);
}
