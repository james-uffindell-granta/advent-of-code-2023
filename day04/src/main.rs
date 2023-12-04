use std::collections::{HashSet, HashMap};

use nom::{
    bytes::complete::tag,
    character::complete as cc,
    combinator::all_consuming,
    multi::separated_list1,
    sequence::{separated_pair, tuple},
    Finish,
    IResult,
};

#[derive(Debug, Clone)]
pub struct Card {
    number: u64,
    winning_numbers: Vec<u64>,
    chosen_numbers: Vec<u64>,
}

impl Card {
    pub fn num_matches(&self) -> usize {
        let winning_numbers = self.winning_numbers.iter().collect::<HashSet<_>>();
        let chosen_numbers = self.chosen_numbers.iter().collect::<HashSet<_>>();
        winning_numbers.intersection(&chosen_numbers).count()
    }

    pub fn points(&self) -> u64 {
        let count = self.num_matches();
        if count == 0 {
            0
        } else {
            2_u64.pow(count as u32 - 1)
        }
    }
}

pub fn parse_card(input: &str) -> IResult<&str, Card> {
    let (rest,
        (_, _, number, _, _,
            (winning_numbers, chosen_numbers))) =
        tuple((tag("Card"), cc::space1, cc::u64, tag(":"), cc::space1,
        separated_pair(
            separated_list1(cc::space1, cc::u64),
            tuple((cc::space1, tag("|"), cc::space1)),
            separated_list1(cc::space1, cc::u64)),
        ))(input)?;

    Ok((rest, Card { number, winning_numbers, chosen_numbers }))
}

pub fn parse_input(input: &str) -> Vec<Card> {
    let mut cards = Vec::new();
    for line in input.lines() {
        match all_consuming(parse_card)(line).finish() {
            Ok((_, card)) => cards.push(card),
            Err(e) => { dbg!(e); unreachable!() }
        }
    }

    cards
}

pub fn part_1(cards: &[Card]) -> u64 {
    cards.iter().map(|c| c.points()).sum()
}

pub fn part_2(cards: &[Card]) -> u64 {
    let mut number_of_copies = HashMap::new();
    for card in cards {
        *number_of_copies.entry(card.number).or_insert(0) += 1;
        let extra_cards = card.num_matches() as u64;
        let number_of_copies_of_this_card = *number_of_copies.get(&card.number).unwrap();
        for card_number in card.number + 1 ..= card.number + extra_cards {
            *number_of_copies.entry(card_number).or_insert(0) += number_of_copies_of_this_card;
        }
    }

    number_of_copies.values().sum()
}

fn main() {
    let cards = parse_input(include_str!("../input.txt"));
    println!("Part 1: {}", part_1(&cards));
    println!("Part 2: {}", part_2(&cards));
}

#[test]
pub fn test() {
    let input = r"Card 1: 41 48 83 86 17 | 83 86  6 31 17  9 48 53
Card 2: 13 32 20 16 61 | 61 30 68 82 17 32 24 19
Card 3:  1 21 53 59 44 | 69 82 63 72 16 21 14  1
Card 4: 41 92 73 84 69 | 59 84 76 51 58  5 54 83
Card 5: 87 83 26 28 32 | 88 30 70 12 93 22 82 36
Card 6: 31 18 13 56 72 | 74 77 10 23 35 67 36 11";

    let cards = dbg!(parse_input(input));
    assert_eq!(part_1(&cards), 13);
    assert_eq!(part_2(&cards), 30);
}