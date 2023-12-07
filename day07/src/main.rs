use std::collections::HashSet;

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug, Hash)]
pub enum Card {
    Joker, Two, Three, Four, Five, Six, Seven, Eight, Nine, Ten, Jack, Queen, King, Ace
}

impl From<char> for Card {
    fn from(value: char) -> Self {
        match value {
            '2' => Self::Two,
            '3' => Self::Three,
            '4' => Self::Four,
            '5' => Self::Five,
            '6' => Self::Six,
            '7' => Self::Seven,
            '8' => Self::Eight,
            '9' => Self::Nine,
            'T' => Self::Ten,
            'J' => Self::Jack,
            'Q' => Self::Queen,
            'K' => Self::King,
            'A' => Self::Ace,
            _ => unreachable!(),
        }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, Debug)]
pub enum HandType {
    HighCard, OnePair, TwoPair, ThreeKind, FullHouse, FourKind, FiveKind
}

#[derive(Clone, Debug, PartialEq, Eq)]
pub struct Hand(Vec<Card>);

impl Hand {
    pub fn hand_type(&self) -> HandType {
        let unique_cards = self.0.iter().collect::<HashSet<_>>();
        let most_identical_cards = unique_cards.iter().map(|c| self.0.iter().filter(|card| c == card).count()).max().unwrap();
        match (unique_cards.len(), most_identical_cards) {
            // these three cases are uniquely determined by how many unique cards there are
            (5, _) => HandType::HighCard,
            (4, _) => HandType::OnePair,
            (1, _) => HandType::FiveKind,
            // these have two possibilities each
            (2, 4) => HandType::FourKind,
            (2, 3) => HandType::FullHouse,
            (3, 3) => HandType::ThreeKind,
            (3, 2) => HandType::TwoPair,
            _ => unreachable!(),
        }
    }
}

impl From<&str> for Hand {
    fn from(value: &str) -> Self {
        Self(value.chars().map(Card::from).collect())
    }
}

impl Ord for Hand {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // first compare hand type, then the list of cards
        self.hand_type().cmp(&other.hand_type()).then(self.0.cmp(&other.0))
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// wrapper that implements joker logic
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct JokerHand(Vec<Card>);

impl JokerHand {
    pub fn hand_type(&self) -> HandType {
        let literal_type = Hand(self.0.clone()).hand_type();
        let number_jokers = self.0.iter().filter(|&c| c == &Card::Joker).count();

        match (literal_type, number_jokers) {
            // nothing we can do in these situations
            (HandType::FiveKind, _) => HandType::FiveKind,
            (hand_type, 0) => hand_type,
            // we have some jokers and some room to play with
            // four-kinds can be promoted - either the four cards are jokers, or the other is
            (HandType::FourKind, _) => HandType::FiveKind,
            // similarly: either the three cards are jokers, or the two are
            (HandType::FullHouse, _) => HandType::FiveKind,
            // either the three cards are jokers, or one of the others is => we can make a four
            (HandType::ThreeKind, _) => HandType::FourKind,
            // two pairs: if one of the pairs is jokers, we can make a four
            // but if the remaining card is a joker, we can make a full house
            (HandType::TwoPair, 2) => HandType::FourKind,
            (HandType::TwoPair, 1) => HandType::FullHouse,
            // either the pair is jokers, or one of the others is => we can make a three
            (HandType::OnePair, _) => HandType::ThreeKind,
            // all five cards are different, the best we can do is a pair
            (HandType::HighCard, _) => HandType::OnePair,
            _ => unreachable!(),
        }
    }
}

impl From<&Hand> for JokerHand {
    fn from(value: &Hand) -> Self {
        // convert any Jacks to Jokers
        Self(value.0.iter().map(|c| if c == &Card::Jack { Card::Joker } else { *c }).collect())
    }
}

impl Ord for JokerHand {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // first compare hand type, then the list of cards
        self.hand_type().cmp(&other.hand_type()).then(self.0.cmp(&other.0))
    }
}

impl PartialOrd for JokerHand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub fn parse_input(input: &str) -> Vec<(Hand, u64)> {
    input.lines().map(|line| {
        let (hand, bid) = line.split_once(' ').unwrap();
        (hand.into(), bid.parse().unwrap())
    }).collect()
}

pub fn part_1(games: &[(Hand, u64)]) -> u64 {
    let mut games = games.to_vec();
    games.sort_by_key(|(h, _)| h.clone());
    games.iter().enumerate().map(|(rank, (_, bid))| (rank as u64 + 1) * *bid).sum()
}

pub fn part_2(games: &[(Hand, u64)]) -> u64 {
    let mut joker_games = games.iter().map(|(h, b)| (JokerHand::from(h), b)).collect::<Vec<_>>();
    joker_games.sort_by_key(|(jh, _)| jh.clone());
    joker_games.iter().enumerate().map(|(rank, (_, bid))| (rank as u64 + 1) * *bid).sum()
}

fn main() {
    let input = include_str!("../input.txt");
    let games = parse_input(input);
    println!("Part 1: {}", part_1(&games));
    println!("Part 2: {}", part_2(&games));
}

#[test]
pub fn test() {
    let input = r"32T3K 765
T55J5 684
KK677 28
KTJJT 220
QQQJA 483";

    let games = parse_input(input);
    assert_eq!(part_1(&games), 6440);
    assert_eq!(part_2(&games), 5905);
}