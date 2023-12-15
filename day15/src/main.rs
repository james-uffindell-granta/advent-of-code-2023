use std::collections::HashMap;

pub fn hash(input: &str) -> u64 {
    input.bytes().fold(0, |acc, b| ((acc + b as u64) * 17) % 256)
}

pub fn part_1(input: &str) -> u64 {
    input.split(',').map(hash).sum()
}

pub enum Instruction<'a> {
    Insert { label: &'a str, focal_length: u64, },
    Remove { label: &'a str }
}

impl<'a> From<&'a str> for Instruction<'a> {
    fn from(value: &'a str) -> Self {
        if let Some((label, focal_length)) = value.split_once('=') {
            Self::Insert { label, focal_length: focal_length.parse().unwrap(), }
        } else {
            assert!(value.ends_with('-'));
            Self::Remove { label: &value[.. value.len() - 1]}
        }
    }
}

pub fn part_2(input: &str) -> u64 {
    let mut map = HashMap::new();
    for step in input.split(',') {
        match Instruction::from(step) {
            Instruction::Insert { label, focal_length } => {
                let box_contents = map.entry(hash(label)).or_insert(Vec::new());
                if let Some((_, lens)) = box_contents.iter_mut().find(|(l, _)| l == &label) {
                    *lens = focal_length;
                } else {
                    box_contents.push((label, focal_length));
                }
            },
            Instruction::Remove { label } => {
                let box_contents = map.entry(hash(label)).or_insert(Vec::new());
                if let Some(index) = box_contents.iter().position(|(l, _)| l == &label) {
                    box_contents.remove(index);
                }
            },
        }
    }

    map.into_iter().flat_map(
        |(box_number, lenses)| 
            lenses.into_iter().enumerate()
                .map(move |(index, (_, lens))| (box_number + 1) * (index as u64 + 1) * lens)
    ).sum()
}

fn main() {
    let input = include_str!("../input.txt");
    println!("Part 1: {}", part_1(input));
    println!("Part 2: {}", part_2(input));
}

#[test]
pub fn test() {
    assert_eq!(part_1("HASH"), 52);
    let input = r"rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7";
    assert_eq!(part_1(input), 1320);
    assert_eq!(part_2(input), 145);
}
