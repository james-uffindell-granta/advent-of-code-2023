use std::fs::File;
use std::io::Write;

use itertools::Itertools;

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
        if self.0.iter().all_equal() {
            return HandType::FiveKind;
        } else if self.0.iter().all_unique() {
            return HandType::HighCard;
        }

        let unique_cards = self.0.iter().unique().collect::<Vec<_>>();
        if unique_cards.len() == 4 {
            return HandType::OnePair;
        } else {
            let most = unique_cards.iter().map(|c| self.0.iter().filter(|card| c == card).count()).max().unwrap();
            if most == 4 {
                return HandType::FourKind;
            } else if most == 3 {
                if unique_cards.len() == 2 {
                    return HandType::FullHouse;
                } else {
                    return HandType::ThreeKind;
                }
            } else if most == 2 {
                return HandType::TwoPair;
            } else {
                unreachable!();
            }
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
        self.hand_type().cmp(&other.hand_type()).then(self.0.cmp(&other.0))
    }
}

impl PartialOrd for Hand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

// interesting: defining new comparator on a Card wrapper and using a vec of that didn't work?!
// look into why later

// wrapper that treats J as Joker instead of Jack
#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct JokerHand(Vec<Card>);

impl JokerHand {
    pub fn hand_type(&self) -> HandType {
        // This case is still easy - jokers or not, there's nothing we can do
        if self.0.iter().all_equal() {
            return HandType::FiveKind;
        }
        
        let number_jokers = self.0.iter().filter(|&c| c == &Card::Joker).count();

        if self.0.iter().all_unique() {
            // we have a "logical" high card
            if number_jokers == 0 {
                // if no jokers, nothing to do
                return HandType::HighCard;
            } else if number_jokers == 1 {
                // one joker: combine it with one of the cards
                return HandType::OnePair;
            } else {
                unreachable!();
            }
        }

        let unique_cards = self.0.iter().unique().collect::<Vec<_>>();

        // we have four different cards in our hand, so only two the same
        if unique_cards.len() == 4 {
            // this is a 'logical' single pair
            if number_jokers == 0 {
                // without jokers no change
                return HandType::OnePair;
            } else if number_jokers == 1 || number_jokers == 2 {
                // we have a pair of jokers and three other cards
                // best to make a three again
                return HandType::ThreeKind;
            } else {
                unreachable!();
            }
        } else {
            let most = unique_cards.iter().map(|c| self.0.iter().filter(|card| c == card).count()).max().unwrap();
            if most == 4 {
                // we have four of some sort of card
                if number_jokers == 0 {
                    // as before
                    return HandType::FourKind;
                } else if number_jokers == 1 || number_jokers == 4 {
                    // we have four of some card and one other card, and at least one joker.
                    // if it's four jokers, make them all the same => five; if it's one joker,
                    // do the same
                    return HandType::FiveKind;
                } else {
                    unreachable!();
                }
            } else if most == 3 {
                // at most three of some card
                if unique_cards.len() == 2 {
                    if number_jokers == 0 {
                        return HandType::FullHouse;
                    } else {
                        // we had a "full house" with jokers in, so we can convert that to a five
                        // by turning the jokers into whatever the other card was
                        return HandType::FiveKind;
                    }
                } else {
                    // we have a 'three of a kind' with jokers in
                    if number_jokers == 0 {
                        return HandType::ThreeKind;
                    } else if number_jokers == 1 {
                        // can't be in the three - so promote the three to a four
                        return HandType::FourKind;
                    } else if number_jokers == 2 {
                        // this would have been a full house which we already handled
                        unreachable!();
                    } else if number_jokers == 3 {
                        // the jokers are the three, but the other two cards must be different
                        // best we can do is four
                        return HandType::FourKind;
                    } else {
                        unreachable!();
                    }
                }
            } else if most == 2 {
                // we have a logical two pair
                if number_jokers == 0 {
                    return HandType::TwoPair;
                } else if number_jokers == 1 {
                    // we have one joker and two pairs, so the joker wasn't one of the pairs
                    // so we can make a full house
                    return HandType::FullHouse;
                } else if number_jokers == 2 {
                    // we have two pairs, and one of them was a pair of jokers - best to make a four
                    return HandType::FourKind;
                } else {
                    unreachable!();
                }
            } else {
                unreachable!();
            }
        }
    }
}

impl From<&Hand> for JokerHand {
    fn from(value: &Hand) -> Self {
        Self(value.0.iter().map(|c| if c == &Card::Jack { Card::Joker } else { *c }).collect())
    }
}

impl Ord for JokerHand {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.hand_type().cmp(&other.hand_type()).then(self.0.cmp(&other.0))
    }
}

impl PartialOrd for JokerHand {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

pub fn parse_input(input: &str) -> Vec<(Hand, u64)> {
    let mut games = Vec::new();
    for line in input.lines() {
        let (hand, bid) = line.split_once(' ').unwrap();
        games.push((hand.into(), bid.parse().unwrap()));
    }

    games
}

pub fn part_1(games: &[(Hand, u64)]) -> u64 {
    let mut games = games.to_vec();
    games.sort_by_key(|(h, _)| h.clone());
    // let mut file = File::create("sorted_games.txt").unwrap();
    // for game in &games {
    //     writeln!(file, "{:?}", game);
    // }

    // dbg!(&games);
    games.iter().enumerate().map(|(rank, (_, bid))| (rank as u64 + 1) * *bid).sum()
}

pub fn part_2(games: &[(Hand, u64)]) -> u64 {
    let mut joker_games = games.iter().map(|(h, b)| (JokerHand::from(h), b)).collect::<Vec<_>>();
    joker_games.sort_by_key(|(jh, _)| jh.clone());
    // let mut file = File::create("sorted_games.txt").unwrap();
    // for game in &joker_games {
    //     writeln!(file, "{:?}", game);
    // }

    // dbg!(&joker_games);
    joker_games.iter().enumerate().map(|(rank, (_, bid))| (rank as u64 + 1) * *bid).sum()
}

fn main() {
    let input = include_str!("../input.txt");
    let games = parse_input(input);
    println!("Part 1: {}", part_1(&games));
    println!("Part 2: {}", part_2(&games));
}

#[test]
pub fn test_ordering() {
    // (JokerHand([JokerCard(Jack), JokerCard(Three), JokerCard(Jack), JokerCard(Jack), JokerCard(Three)]), 66)
// (JokerHand([JokerCard(Jack), JokerCard(Jack), JokerCard(King), JokerCard(King), JokerCard(King)]), 937)
// (JokerHand([JokerCard(Jack), JokerCard(Jack), JokerCard(Jack), JokerCard(Jack), JokerCard(Jack)]), 219)

    let one = JokerHand::from(&Hand::from("J3JJ3"));
    let two = JokerHand::from(&Hand::from("JJKKK"));
    // assert!(two < one); // same first card, but joker is less than 3
    // assert!(!(one < two));

    assert_eq!(one.hand_type(), HandType::FiveKind);
    assert_eq!(two.hand_type(), HandType::FiveKind);

    // assert!(JokerCard(Card::Jack) < JokerCard(Card::Three));
    // assert!(JokerCard(Card::Three) < JokerCard(Card::Jack));
    dbg!(&one.0);
    dbg!(&two.0);
    assert!(one.0 < two.0);
    assert!(two.0 < one.0);
}

#[test]
pub fn test() {
    let input = r"32T3K 765
T55J5 684
KK677 28
KTJJT 220
QQQJA 483";

    let games = dbg!(parse_input(input));
    assert_eq!(part_1(&games), 6440);
    assert_eq!(part_2(&games), 5905);
}