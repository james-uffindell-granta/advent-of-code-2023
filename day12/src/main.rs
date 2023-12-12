use std::collections::HashMap;

#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Line {
    string_part: String,
    numbers: Vec<u64>,
}

impl Line {
    pub fn expand(&self) -> Line {
        Line {
            string_part: format!("{}?{}?{}?{}?{}", self.string_part.clone(), self.string_part.clone(), self.string_part.clone(), self.string_part.clone(), self.string_part.clone()),
            numbers: self.numbers.repeat(5),
        }
    }

    // hashmap for memoizing the answers we see along the way - without this it's ungodly slow
    pub fn count_options(&self, answers: &mut HashMap<(String, Vec<u64>), usize>) -> usize {
        Self::find_fill_options(answers, &self.string_part, &self.numbers)
    }

    pub fn find_fill_options<'a>(answers: &mut HashMap<(String, Vec<u64>), usize>, remaining_string: &'a str, remaining_numbers: &'a [u64]) -> usize {
        // first off: if we already know the answer because we've seen this combination of string-and-numbers before, return it.
        let current_state = (remaining_string.to_owned(), remaining_numbers.to_vec());
        if let Some(answer) = answers.get(&current_state) {
            return *answer;
        }

        // otherwise, check if we're out of numbers to fill, and see if that makes sense for this string.
        if remaining_numbers.is_empty() && remaining_string.contains('#') {
            // if there are no more numbers left, but the remaining string has a # in, we've gone wrong => no answers.
            answers.insert(current_state, 0);
            return 0;
        } else if remaining_numbers.is_empty() {
            // if there are no more numbers left, and the remaining string has no # in, there's only one solution (replace all ? with .).
            // this also covers the case where the remaining string is empty.
            answers.insert(current_state, 1);
            return 1;
        }

        // so we have at least one number still to try and fill in to the string - do we have space for it?
        if remaining_string.len() < remaining_numbers.iter().sum::<u64>() as usize + remaining_numbers.len() - 1 {
            // if our remaining numbers (plus the mandatory . between them) would take up more space than we have left in the string
            // then we've gone wrong => no answers.
            answers.insert(current_state, 0);
            return 0;
        }

        // we have at least one number to fit in, and space to fit it - let's see what the next char in the string is.
        match remaining_string.chars().next().unwrap() {
            '.' => {
                // we can't fit the number here at the start since that's a . - skip ahead to the next ? or # and try again there
                // if there aren't any more ? or # then there aren't any answers (since we know we have a number to fit in)
                let answer = match remaining_string.find(['?', '#']) {
                    Some(non_dot) => Self::find_fill_options(answers, &remaining_string[non_dot..], remaining_numbers),
                    None => 0,
                };
                answers.insert(current_state, answer);
                answer
            },
            '#' => {
                // we already checked at the top: we have at least one number to fill in, and there is space for it
                // since we have a # at the start, it must go here at the start - so check the next <number> chars to make sure they're all ? or #
                let next_number = remaining_numbers[0] as usize;
                if remaining_string.chars().take(next_number).any(|c| c == '.') {
                    // found a .: this number can't go here after all, contradiction => no answers.
                    answers.insert(current_state, 0);
                    return 0;
                }

                // so the <next number> chars are either # or ?, and so we can fit the next number in here.
                // skip ahead that many chars in the string (these will all become #)
                // and skip ahead one number in the 'remaining numbers' list too
                let new_remaining_string = &remaining_string[next_number..];
                let new_remaining_numbers = &remaining_numbers[1..];

                let answer = if new_remaining_string.is_empty() {
                    // if we're now out of string, there's either one answer (no numbers left either), or no answer (still some numbers left)
                    if new_remaining_numbers.is_empty() { 1 } else { 0 }
                } else if new_remaining_string.starts_with('#') {
                    // we just put a number here in so the next character has to be able to be a .
                    // but the string says it has to be a # => contradiction, no answers.
                    0
                } else {
                    // otherwise, we can make the next character a ., so skip it and recurse
                    // try to fit the new smaller list of numbers into the new smaller string.
                    Self::find_fill_options(answers, &new_remaining_string[1..], new_remaining_numbers)
                };

                answers.insert(current_state, answer);
                answer
            },
            '?' => {
                // we have two choices for the ? char, we can either make it a . or a #
                // just try both of them and add up the possibilities for each 
                let dot_option = remaining_string.replacen('?', ".", 1);
                let hash_option = remaining_string.replacen('?', "#", 1);
                let answer = Self::find_fill_options(answers, &dot_option, remaining_numbers)
                    + Self::find_fill_options(answers, &hash_option, remaining_numbers);
                answers.insert(current_state, answer);
                answer
                
            },
            _ => unreachable!(),
        }
    }
}

pub fn parse_input(input: &str) -> Vec<Line> {
    input.lines().map(|line| {
        let (code, key) = line.split_once(' ').unwrap();
        let numbers = key.split(',').map(|n| n.parse().unwrap()).collect();
        Line { string_part: code.to_owned(), numbers }
    }).collect()
}

pub fn part_1(lines: &[Line], answers: &mut HashMap<(String, Vec<u64>), usize>) -> usize {
    lines.iter().map(|line| line.count_options(answers)).sum()
}

pub fn part_2(lines: &[Line], answers: &mut HashMap<(String, Vec<u64>), usize>) -> usize {
    lines.iter().map(|line| line.expand().count_options(answers)).sum()
}

fn main() {
    let input = include_str!("../input.txt");
    let lines = parse_input(input); 
    let mut answers = HashMap::new();
    println!("Part 1: {}", part_1(&lines, &mut answers));
    println!("Part 2: {}", part_2(&lines, &mut answers));
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
    assert_eq!(part_1(&lines, &mut answers), 21);
    assert_eq!(part_2(&lines, &mut answers), 525152);
}