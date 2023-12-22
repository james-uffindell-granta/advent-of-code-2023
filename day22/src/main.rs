use std::{collections::{HashSet, HashMap, BTreeMap, BTreeSet}, ops::Add};
use itertools::Itertools;

// #[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
// pub struct Coord2 {
//     x: i64, y: i64
// }

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Coord3 {
    x: i64, y: i64, z: i64
}

impl From<(i64, i64, i64)> for Coord3 {
    fn from((x, y, z): (i64, i64, i64)) -> Self {
        Self { x, y, z }
    }
}

impl Add<(i64, i64, i64)> for Coord3 {
    type Output = Coord3;

    fn add(self, (x, y, z): (i64, i64, i64)) -> Self::Output {
        (self.x + x, self.y + y, self.z + z).into()
    }
}

impl Add<Coord3> for Coord3 {
    type Output = Coord3;

    fn add(self, Coord3 { x, y, z }: Coord3) -> Self::Output {
        (self.x + x, self.y + y, self.z + z).into()
    }
}

// impl Coord3 {
//     pub fn shadow(&self) -> Coord2 {
//         Coord2 { x: self.x, y: self.y}
//     }
// }

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct Brick {
    start: Coord3,
    end: Coord3,
}

impl Brick {
    pub fn new(start: Coord3, end: Coord3) -> Self {
        // normalize so the 'smaller' end is the start
        if start <= end {
            Self { start, end }
        } else {
            Self { start: end, end: start }
        }
    }

    pub fn shadow_at(&self, z: i64) -> HashSet<Coord3> {
        (self.start.x ..= self.end.x)
            .flat_map(|x| (self.start.y ..= self.end.y).map(move |y| (Coord3 { x, y, z })))
            .collect()
    }

    pub fn extent(&self) -> HashSet<Coord3> {
        (self.start.x ..= self.end.x)
            .flat_map(move |x| (self.start.y ..= self.end.y)
                .flat_map(move |y| (self.start.z ..= self.end.z).map(move |z| Coord3 { x, y, z })))
            .collect()
    }

    pub fn translate_by(&self, translation: (i64, i64, i64)) -> Brick {
        Brick::new(self.start + translation, self.end + translation)
    }
}

#[derive(Clone, Debug)]
pub struct Stack {
    bricks: HashMap<usize, Brick>,
}

#[derive(Clone, Debug)]
pub struct SettledStack {
    settled_coords: HashMap<Coord3, usize>,
    supported_by: HashMap<usize, HashSet<usize>>,
}

impl Stack {
    pub fn drop_bricks(&self) -> SettledStack {
        let mut bricks_by_starting_z = self.bricks.iter().collect::<Vec<_>>();
        bricks_by_starting_z.sort_by_key(|(_, b)| b.start.z);
        // let bricks_by_starting_z = self.bricks.iter()
        //     .map(|(index, b)| (b.start.z, index))
        //     .collect::<BTreeMap<_, _>>();

        // map of "settled coord" to "brick they were part of"
        let mut settled_coords = HashMap::new();
        // map of "settled brick" to "bricks they are supported by"
        let mut supported_by = HashMap::new();

        for (brick_number, brick) in bricks_by_starting_z {
            let brick = self.bricks.get(&brick_number).unwrap();
            // if brick.start.z == 0 {
            //     // can't drop this brick any further - it must be settled already
            //     supported_by.insert(*brick_number, HashSet::new());
            //     for c in brick.extent() {
            //         settled_coords.insert(c, *brick_number);
            //     }
            // } else 
            if brick.start.z <= 0 {
                // gone wrong somewhere
                unreachable!();
            } else {
                let starting_height = brick.start.z;
                let mut will_rest_at = starting_height;
                let settled = settled_coords.keys().copied().collect::<HashSet<_>>();
                for resting_height in (1 ..= starting_height - 1).rev() {
                    if brick.shadow_at(resting_height).intersection(&settled).next().is_some() {
                        // found a blocker
                        break;
                    }

                    will_rest_at = resting_height;
                }

                // book-keeping:
                // now that we know where this brick is settling, keep track of what's supporting it
                if will_rest_at == 1 {
                    supported_by.insert(*brick_number, HashSet::new());
                } else {
                    let supporting_coords = brick.shadow_at(will_rest_at - 1);
                    let supporting_bricks = supporting_coords.iter()
                        .filter_map(|c| settled_coords.get(c))
                        .copied()
                        .collect();
                    supported_by.insert(*brick_number, supporting_bricks);
                }

                // then either way we need to add this brick to the settled bricks
                let settled_brick = brick.translate_by((0, 0, -(starting_height - will_rest_at)));
                for c in settled_brick.extent() {
                    settled_coords.insert(c, *brick_number);
                }
            }
        }

        SettledStack { settled_coords, supported_by }
    }
}

impl SettledStack {

    pub fn bricks_directly_supported_by(&self, bricks: &BTreeSet<usize>) -> BTreeSet<usize> {
        self.supported_by.iter()
            .filter(|(_, supp)| 
                !supp.is_empty() && supp.iter().all(|b| bricks.contains(b)))
            .map(|(b, _)| *b)
            .collect()
    }

    pub fn dependent_brick_count(&self, bricks_to_remove: &BTreeSet<usize>,
        answers: &mut HashMap<BTreeSet<usize>, usize>
    ) -> usize {
        // println!("Deleting bricks {:?}", bricks_to_remove);
        if let Some(answer) = answers.get(bricks_to_remove) {
            return *answer;
        }

        // otherwise we haven't figured out how many for this one yet
        let bricks_directly_supported = self.bricks_directly_supported_by(bricks_to_remove);
        // println!("Bricks directly supported by these bricks: {:?}", bricks_directly_supported);
        let extra_bricks_supported = bricks_directly_supported.difference(bricks_to_remove).copied().collect::<BTreeSet<_>>();
        let extra_bricks_deleted = extra_bricks_supported.len();
        // println!("{} new bricks directly supported by these bricks: {:?}", extra_bricks_deleted, extra_bricks_supported);

        // deleting this didn't affect anything
        if extra_bricks_deleted == 0 {
            let result = 0;
            answers.insert(bricks_to_remove.clone(), result);
            return result;
        }

        let all_bricks_deleted_so_far = extra_bricks_supported.union(bricks_to_remove).copied().collect::<BTreeSet<_>>();

        let dependent_bricks = self.dependent_brick_count(&all_bricks_deleted_so_far, answers);
        let result = extra_bricks_deleted + dependent_bricks;
        answers.insert(bricks_to_remove.clone(), result);
        result
    }

    // returns, for each brick, the 
    // pub fn get_transitive_support(&self) -> HashMap<usize, HashSet<usize>> {

    // }
}

pub fn parse_input(input: &str) -> Stack {
    let mut bricks = HashMap::new();
    for (brick_number, line) in input.lines().enumerate() {
        let (start, end) = line.split_once('~').unwrap();
        let s @ (_, _, _) = start.split(',')
            .map(|s| s.parse().unwrap())
            .collect_tuple().unwrap();
        let e @ (_, _, _) = end.split(',')
            .map(|s| s.parse().unwrap())
            .collect_tuple().unwrap();
        bricks.insert(brick_number, Brick::new(s.into(), e.into()));
    }

    Stack { bricks }
}

pub fn part_1(stack: &Stack) -> usize {
    let settled = stack.drop_bricks();
    // find the bricks that are the sole supporters of other bricks
    let all_bricks = settled.supported_by.keys().collect::<HashSet<_>>();
    let single_supports = settled.supported_by.values()
        .filter(|&set| set.len() == 1)
        .map(|set| set.iter().next().unwrap())
        .collect::<HashSet<_>>();
    all_bricks.len() - single_supports.len()
}

pub fn part_2(stack: &Stack) -> usize {
    let settled = stack.drop_bricks();
    
    let mut answers = HashMap::new();
    let bricks = settled.supported_by.keys().copied().collect::<HashSet<_>>();
    let mut total = 0;
    for brick in bricks {
        let set = BTreeSet::from([brick]);
        let deleted = settled.dependent_brick_count(&set, &mut answers);
        total += deleted;
    }

    total
}

fn main() {
    let input = include_str!("../input.txt");
    let stack = parse_input(input);
    println!("Part 1: {}", part_1(&stack));
    println!("Part 2: {}", part_2(&stack));
}

#[test]
pub fn test() {
    let input = r"1,0,1~1,2,1
0,0,2~2,0,2
0,2,3~2,2,3
0,0,4~0,2,4
2,0,5~2,2,5
0,1,6~2,1,6
1,1,8~1,1,9";

    let bricks = parse_input(input);
    let settled = bricks.drop_bricks();
    dbg!(&settled);
    dbg!(part_1(&bricks));

    let mut answers = HashMap::new();
    let brick_to_delete = BTreeSet::from([0usize]);
    dbg!(settled.dependent_brick_count(&BTreeSet::from([0usize]), &mut answers));
    dbg!(settled.dependent_brick_count(&BTreeSet::from([1usize]), &mut answers));
    dbg!(part_2(&bricks));
}