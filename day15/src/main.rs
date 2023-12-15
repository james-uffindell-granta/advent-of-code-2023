use std::collections::HashMap;

pub fn hash(input: &str) -> u64 {
    input.bytes().fold(0, |acc, b| ((acc + b as u64) * 17) % 256)
}

pub fn part_1(input: &str) -> u64 {
    input.split(',').map(hash).sum()
}

pub fn part_2(input: &str) -> u64 {
    let mut map : HashMap<u64, Vec<(&str, u64)>> = HashMap::new();
    for step in input.split(',') {
        if let Some((label, focal_length)) = step.split_once('=') {
            let new_lens = focal_length.parse().unwrap();
            let box_contents = map.entry(hash(label)).or_insert(Vec::new());
            if let Some((_, lens)) = box_contents.iter_mut().find(|(l, _)| l == &label) {
                *lens = new_lens;
            } else {
                box_contents.push((label, new_lens));
            }
        } else {
            // the dash
            assert!(step.ends_with('-'));
            let label = step.replace('-', "");
            let box_contents = map.entry(hash(&label)).or_insert(Vec::new());
            if let Some(index) = box_contents.iter().position(|(l, _)| l == &label) {
                box_contents.remove(index);
            }
        }

        // dbg!(&map);
    }

    let mut focusing_power = 0;
    for (box_number, lenses) in map {
        for (index, (_, lens)) in lenses.iter().enumerate() {
            let lens_power = (box_number + 1) * (index as u64 + 1) * *lens;
            focusing_power += lens_power;
        }
    }

    focusing_power

}

fn main() {
    let input = include_str!("../input.txt");
    println!("Part 1: {}", part_1(input));
    println!("Part 2: {}", part_2(input));
}

#[test]
pub fn test() {
    let input = r"rn=1,cm-,qp=3,cm=2,qp-,pc=4,ot=9,ab=5,pc-,pc=6,ot=7";
    assert_eq!(part_1(input), 1320);

    assert_eq!(part_1("HASH"), 52);

    assert_eq!(part_2(input), 145);
}
