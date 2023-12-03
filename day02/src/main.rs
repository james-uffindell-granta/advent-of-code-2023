use std::collections::HashMap;
use nom::{
    branch::alt,
    bytes::complete::tag,
    character::complete as cc,
    combinator::{all_consuming, map},
    multi::separated_list1,
    sequence::{separated_pair, tuple},
    Finish,
    IResult,
};

#[derive(PartialEq, Eq, Hash, Copy, Debug, Clone)]
pub enum Color {
    Red,
    Green,
    Blue,
}

#[derive(Clone, Debug)]
pub struct Game {
    number: usize,
    max_seen: HashMap<Color, usize>,
}

impl Game {
    pub fn new(number: usize) -> Self {
        Self { number, max_seen: HashMap::new() }
    }

    pub fn record_play(&mut self, cubes_pulled: &HashMap<Color, usize>){
        for (color, number) in cubes_pulled {
            let seen_so_far = self.max_seen.entry(*color).or_insert(0);
            *seen_so_far = (*seen_so_far).max(*number);
        }
    }

    pub fn possible_with(&self, cubes_available: &HashMap<Color, usize>) -> bool {
        self.max_seen.iter().all(|(c, n)| cubes_available.get(c).unwrap_or(&0) >= n)
    }
}

pub fn parse_color(input: &str) -> IResult<&str, Color> {
    alt((
        map(tag("red"), |_| Color::Red),
        map(tag("green"), |_| Color::Green),
        map(tag("blue"), |_| Color::Blue)))(input)
}

pub fn parse_play(input: &str) -> IResult<&str, HashMap<Color, usize>> {
    map(
        separated_list1(
            tag(", "),
            separated_pair(cc::u64, tag(" "), parse_color)),
        |pairs| pairs.into_iter().map(|(n, c)| (c, n as usize)).collect())
    (input)
}

pub fn parse_game(input: &str) -> IResult<&str, Game> {
    let (rest, (_, number, _)) = tuple((tag("Game "), cc::u64, tag(": ")))(input)?;
    let mut game = Game::new(number as usize);

    let (rest, _) = separated_list1(
        tag("; "),
        map(parse_play, |play| {
            game.record_play(&play);
        })
    )(rest)?;
    Ok((rest, game))
}

pub fn parse_input(input: &str) -> Vec<Game> {
    let mut games = Vec::new();
    for line in input.lines() {
        if let Ok((_, game)) = all_consuming(parse_game)(line).finish() {
            games.push(game);
        } else {
            unreachable!();
        }
    }

    games
}

pub fn part_1(games: &[Game]) -> usize {
    let cubes_available = vec![(Color::Red, 12), (Color::Green, 13), (Color::Blue, 14)].into_iter().collect();
    games.iter()
        .filter(|g| g.possible_with(&cubes_available))
        .map(|g| g.number)
        .sum()
}

pub fn part_2(games: &[Game]) -> usize {
    games.iter()
        .map(|g| {
            let counts = g.max_seen.values().copied().collect::<Vec<_>>();
            if counts.len() != 3 {
                // one of the colors wasn't seen - so the min is 0 and the power is 0
                0
            } else {
                counts.iter().product()
            }
        }).sum()
}

fn main() {
    let input = include_str!("../input.txt");
    let games = parse_input(input);
    println!("Part 1: {}", part_1(&games));
    println!("Part 2: {}", part_2(&games));
}

#[test]
pub fn test() {
    let input = r"Game 1: 3 blue, 4 red; 1 red, 2 green, 6 blue; 2 green
Game 2: 1 blue, 2 green; 3 green, 4 blue, 1 red; 1 green, 1 blue
Game 3: 8 green, 6 blue, 20 red; 5 blue, 4 red, 13 green; 5 green, 1 red
Game 4: 1 green, 3 red, 6 blue; 3 green, 6 red; 3 green, 15 blue, 14 red
Game 5: 6 red, 1 blue, 3 green; 2 blue, 1 red, 2 green";

    let games = parse_input(input);
    assert_eq!(part_1(&games), 8);
    assert_eq!(part_2(&games), 2286);
}
