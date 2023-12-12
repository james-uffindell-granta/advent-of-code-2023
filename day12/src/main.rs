use std::collections::{HashSet, HashMap};
use itertools::{EitherOrBoth, Itertools};
use std::time::Instant;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Line {
    coded_part: String,
    key_part: Vec<u64>,
}

impl Line {
    pub fn expand(&self) -> Line {
        Line {
            coded_part: format!("{}?{}?{}?{}?{}", self.coded_part.clone(), self.coded_part.clone(), self.coded_part.clone(), self.coded_part.clone(), self.coded_part.clone()),
            key_part: self.key_part.repeat(5),
        }
    }

    pub fn count_options<'a>(&'a self, answers: &mut HashMap<(String, Vec<u64>), usize>) -> usize {
        Self::find_fill_options(answers, &self.coded_part, &self.key_part)
    }

    pub fn find_fill_options<'a>(answers: &mut HashMap<(String, Vec<u64>), usize>, remaining_portion: &'a str, remaining_fill: &'a [u64]) -> usize {
        if let Some(answer) = answers.get(&(remaining_portion.to_owned(), remaining_fill.to_vec())) {
            return *answer;
        }

        // quick bail out condition:
        // if we don't have any numbers to put in, but there's a hash, we've gone wrong
        if remaining_fill.is_empty() && remaining_portion.contains('#') {
            answers.insert((remaining_portion.to_owned(), remaining_fill.to_vec()), 0);
            return 0;
        } else if remaining_fill.is_empty() {
            // no more numbers to put in - there's only one way to fill in the rest (all .)
            answers.insert((remaining_portion.to_owned(), remaining_fill.to_vec()), 1);
            return 1;
        }

        // otherwise: if we need to put too many hashes in for the remaining space, we've gone wrong
        // dbg!(remaining_fill);
        let remaining_space_needed = remaining_fill.iter().sum::<u64>() as usize + remaining_fill.len() - 1;
        if remaining_portion.len() < remaining_space_needed {
            answers.insert((remaining_portion.to_owned(), remaining_fill.to_vec()), 0);
            return 0;
        }

        let remaining_hashes_needed = remaining_fill.iter().sum::<u64>() as usize;
        let number_hashes_in_remaining_portion = remaining_portion.chars().filter(|c| *c == '#').count();
        // too many . to fit the hashes in (or too many # to be accounted for by the number we need to fill in)
        if remaining_portion.chars().filter(|c| *c != '.').count() < remaining_hashes_needed
            || number_hashes_in_remaining_portion > remaining_hashes_needed {
            // can't fit the rest of the hashes in at all
            answers.insert((remaining_portion.to_owned(), remaining_fill.to_vec()), 0);
            return 0;
        }

        if remaining_fill.len() == 1 && remaining_fill[0] == 1 {
            // if there's only one number 1 left, and one hash in the string, that must be the only option
            if number_hashes_in_remaining_portion == 1 {
                answers.insert((remaining_portion.to_owned(), remaining_fill.to_vec()), 1);
                return 1;
            }

            // otherwise if there's no hashes in the string, the number of places to put it is number-of-?
            // we know we can't be containing a ? that was actually at the end of a previous block
            // because we'd have consumed it before recursing
            if number_hashes_in_remaining_portion == 0 {
                let answer = remaining_portion.chars().filter(|c| *c == '?').count();
                answers.insert((remaining_portion.to_owned(), remaining_fill.to_vec()), answer);
                return answer;

            }
        }

        // if remaining_portion.chars().all(|c| c == '?') {
        //     let known: u64 = remaining_fill.iter().sum::<u64>() + remaining_fill.len() as u64 - 1;
        //     let num_gaps = remaining_portion.len() as u64 - known;
        //     let num_buckets = remaining_fill.len() + 1;
        //     let answer = generate_bucket_options(num_buckets, num_gaps).len();
        //     answers.insert((remaining_portion.to_owned(), remaining_fill.to_vec()), 0);
        //     return answer;
        // }

        for c in remaining_portion.chars() {
            match c {
                '.' => {
                    // recurse?
                    match remaining_portion.find(['?', '#']) {
                        Some(non_dot) => {
                            let answer = Self::find_fill_options(answers, &remaining_portion[non_dot..], remaining_fill);
                            answers.insert((remaining_portion.to_owned(), remaining_fill.to_vec()), answer);
                            return answer;
                        },
                        None => {
                            let answer = if remaining_fill.is_empty() { 1 } else { 0 };
                            answers.insert((remaining_portion.to_owned(), remaining_fill.to_vec()), answer);
                            return answer;
                        },
                        // return Self::find_fill_options(&remaining_portion[1..], remaining_fill);
                    }
                },
                '#' => {
                    // we have a # at the start of a block
                    if remaining_fill.is_empty() {
                        // contradiction
                        answers.insert((remaining_portion.to_owned(), remaining_fill.to_vec()), 0);
                        return 0;
                    }

                    let next_number = remaining_fill[0] as usize;
                    if remaining_portion.chars().take(next_number).any(|c| c == '.') {
                        // contradiction: we can't fit this number in here
                        answers.insert((remaining_portion.to_owned(), remaining_fill.to_vec()), 0);
                        return 0;
                    }

                    let new_remaining_portion = &remaining_portion[next_number..];
                    if !new_remaining_portion.is_empty() &new_remaining_portion.starts_with('#') {
                        // we need to be able to put a . after it (but it's fine if it's empty)
                        answers.insert((remaining_portion.to_owned(), remaining_fill.to_vec()), 0);
                        return 0;
                    }

                    let new_remaining_fill = &remaining_fill[1..];

                    // two possibilities here
                    if new_remaining_portion.is_empty() && !new_remaining_fill.is_empty() {
                        // not possible
                        answers.insert((remaining_portion.to_owned(), remaining_fill.to_vec()), 0);
                        return 0;
                    } else if new_remaining_portion.is_empty() && new_remaining_fill.is_empty() {
                        // both run out at the same time - this is the one way to fill it in
                        answers.insert((remaining_portion.to_owned(), remaining_fill.to_vec()), 1);
                        return 1;
                    }

                    let new_remaining_portion = &new_remaining_portion[1..];
                    let answer = Self::find_fill_options(answers, new_remaining_portion, new_remaining_fill);
                    answers.insert((remaining_portion.to_owned(), remaining_fill.to_vec()), answer);
                    return answer;
                },
                '?' => {
                    let dot_option = remaining_portion.replacen('?', ".", 1);
                    let hash_option = remaining_portion.replacen('?', "#", 1);
                    let answer = Self::find_fill_options(answers, &dot_option, remaining_fill)
                        + Self::find_fill_options(answers, &hash_option, remaining_fill);
                    answers.insert((remaining_portion.to_owned(), remaining_fill.to_vec()), answer);
                    return answer;
                    
                },
                _ => unreachable!(),
            }
        }

        todo!()
    }

    pub fn find_possibilities(&self) -> usize {
        let known: u64 = self.key_part.iter().sum::<u64>() + self.key_part.len() as u64 - 1;
        let num_gaps = self.coded_part.len() as u64 - known;
        let num_buckets = self.key_part.len() + 1;
        let mut options = 0;
        // println!("Old way: Generating ways for {} gaps in {} buckets", num_gaps, num_buckets);
        let possibilities = generate_bucket_options(num_buckets, num_gaps);
        // dbg!(possibilities.len());
        for mut possibility in possibilities {
            // need to add the extra space into the middle buckets
            for i in 1..possibility.len() - 1 {
                possibility[i] += 1;
            }
            let result = possibility.iter().zip_longest(self.key_part.iter()).flat_map(|v| {
                match v {
                    EitherOrBoth::Both(spaces, hashes) => vec![vec!['.'; *spaces as usize], vec!['#'; *hashes as usize]],
                    EitherOrBoth::Left(spaces) => vec![vec!['.'; *spaces as usize]],
                    EitherOrBoth::Right(_) => unreachable!(), // shouldn't be possible
                }
            }).flatten().collect::<String>();
            // dbg!(&result);
            // dbg!(&self.coded_part);
            if self.coded_part.chars().zip(result.chars()).all(|(c, r)| c == r || c == '?') {
                options += 1;
            }
        }

        options
    }

    pub fn find_possibilities_improved(&self) -> usize {
        let number_known_hashes = self.coded_part.chars().filter(|c| *c == '#').count();
        let number_known_spaces = self.coded_part.chars().filter(|c| *c == '.').count();
        let number_questions = self.coded_part.len() - number_known_hashes - number_known_spaces;

        let number_needed_hashes = self.key_part.iter().sum::<u64>() as usize;
        let number_needed_spaces = self.coded_part.len() - number_needed_hashes;

        let number_missing_hashes = number_needed_hashes - number_known_hashes;
        let number_missing_spaces = number_needed_spaces - number_known_spaces;

        // println!("Generating ways for {} gaps in {} buckets", number_missing_spaces as u64, number_missing_hashes + 1);
        let possibilities = generate_bucket_options(number_missing_hashes + 1, number_missing_spaces as u64);

        // let known: u64 = self.key_part.iter().sum::<u64>() + self.key_part.len() as u64 - 1;
        // let num_gaps = self.coded_part.len() as u64 - known;
        // let num_buckets = self.key_part.len() + 1;
        // let mut options = 0;

        // for mut possibility in generate_bucket_options(num_buckets, num_gaps) {
        //     // need to add the extra space into the middle buckets
        //     for i in 1..possibility.len() - 1 {
        //         possibility[i] += 1;
        //     }
        //     let result = possibility.iter().zip_longest(self.key_part.iter()).flat_map(|v| {
        //         match v {
        //             EitherOrBoth::Both(spaces, hashes) => vec![vec!['.'; *spaces as usize], vec!['#'; *hashes as usize]],
        //             EitherOrBoth::Left(spaces) => vec![vec!['.'; *spaces as usize]],
        //             EitherOrBoth::Right(_) => unreachable!(), // shouldn't be possible
        //         }
        //     }).flatten().collect::<String>();
        //     // dbg!(&result);
        //     // dbg!(&self.coded_part);
        //     if self.coded_part.chars().zip(result.chars()).all(|(c, r)| c == r || c == '?') {
        //         options += 1;
        //     }
        // }

        // options
        0
    }
}

pub fn compare_difficulty(line: &Line) {
    let known: u64 = line.key_part.iter().sum::<u64>() + line.key_part.len() as u64 - 1;
    let num_gaps = line.coded_part.len() as u64 - known;
    let num_buckets = line.key_part.len() + 1;
    let mut options = 0;
    // println!("Old way: Generating ways for {} gaps in {} buckets", num_gaps, num_buckets);

    let number_known_hashes = line.coded_part.chars().filter(|c| *c == '#').count();
    let number_known_spaces = line.coded_part.chars().filter(|c| *c == '.').count();
    let number_questions = line.coded_part.len() - number_known_hashes - number_known_spaces;

    let number_needed_hashes = line.key_part.iter().sum::<u64>() as usize;
    let number_needed_spaces = line.coded_part.len() - number_needed_hashes;

    let number_missing_hashes = number_needed_hashes - number_known_hashes;
    let number_missing_spaces = number_needed_spaces - number_known_spaces;

    // println!("New way: Generating ways for {} gaps in {} buckets", number_missing_spaces as u64, number_missing_hashes + 1);
}

pub fn parse_input(input: &str) -> Vec<Line> {
    input.lines().map(|line| {
        let (code, key) = line.split_once(' ').unwrap();
        let numbers = key.split(',').map(|n| n.parse().unwrap()).collect();
        Line { coded_part: code.to_owned(), key_part: numbers }
    }).collect()
}

pub fn generate_bucket_options(num_buckets: usize, num_gaps: u64) -> HashSet<Vec<u64>> {
    let mut possibilities = HashSet::new();
    // base case
    if num_gaps == 0 {
        possibilities.insert(vec![0; num_buckets]);
    } else {
        for possibility in generate_bucket_options(num_buckets, num_gaps - 1) {
            for b in 0..num_buckets {
                let mut new_possibility = possibility.clone();
                new_possibility[b] += 1;
                possibilities.insert(new_possibility);
            }
        }
    }

    possibilities
}

pub fn part_1(lines: &[Line]) -> usize {
    lines.iter().map(|line| line.find_possibilities()).sum()
}

pub fn part_2(lines: &[Line]) -> usize {
    let mut answers = HashMap::new();
    lines.iter().enumerate().map(|(n, line)| {
        // println!("Processing line {} ({}, {:?})", n, line.coded_part, line.key_part);
        line.expand().count_options(&mut answers)
}   ).sum()
}

fn main() {
    let input = include_str!("../input.txt");
    let lines = parse_input(input); 
    println!("Part 1: {}", part_1(&lines));
    let now = Instant::now();
    println!("Part 2: {}", part_2(&lines));
    println!("Part 2 took: {:2?}", now.elapsed());
}

#[test]
pub fn test() {
    let line = Line { coded_part: "???.###".to_owned(), key_part: vec![1, 1, 3] };
    dbg!(line.find_possibilities());
    let line_2 = Line { coded_part: "?###????????".to_owned(), key_part: vec![3, 2, 1] };
    dbg!(line_2.find_possibilities());
}

#[test]
pub fn test_input() {
    let input = r"???.### 1,1,3
.??..??...?##. 1,1,3
?#?#?#?#?#?#?#? 1,3,1,6
????.#...#... 4,1,1
????.######..#####. 1,6,5
?###???????? 3,2,1";    

    let lines = parse_input(input);
    let mut answers = HashMap::new();
    dbg!(lines[0].count_options(&mut answers));
    dbg!(lines[1].count_options(&mut answers));
    dbg!(lines[2].count_options(&mut answers));
    dbg!(lines[3].count_options(&mut answers));
    dbg!(lines[4].count_options(&mut answers));
    dbg!(lines[5].count_options(&mut answers));
    dbg!(lines[0].expand().count_options(&mut answers));
    dbg!(lines[1].expand().count_options(&mut answers));
    dbg!(lines[2].expand().count_options(&mut answers));
    dbg!(lines[3].expand().count_options(&mut answers));
    dbg!(lines[4].expand().count_options(&mut answers));
    dbg!(lines[5].expand().count_options(&mut answers));
    // assert_eq!(part_1(&lines), 21);
    compare_difficulty(&lines[4]);
    // for line in &lines {
    //     compare_difficulty(&line.expand());
    // }
    // dbg!(lines[0].expand().find_possibilities());
    // dbg!(lines[1].expand().find_possibilities());

    let line = Line { coded_part: "??#????..??.???.".to_owned(), key_part: vec![3, 1, 1, 1] };
    dbg!(line.count_options(&mut answers));
    // dbg!(line.expand().count_options());

    let line2 = Line { coded_part: "???????#???#??????".to_owned(), key_part: vec![1, 1, 7, 1, 1] };
    dbg!(line2.count_options(&mut answers));
    dbg!(line2.expand().count_options(&mut answers));

    let line3 = Line { coded_part: "?#????##??????????".to_owned(), key_part: vec![9, 1, 1, 1] };
    dbg!(line3.count_options(&mut answers));

}

