use std::{collections::HashMap, ops::RangeInclusive};
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::complete as cc,
    combinator::{all_consuming, map},
    multi::separated_list1,
    sequence::tuple,
    Finish,
    IResult,
};

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct RatingRange {
    range: Option<RangeInclusive<u64>>,
}

impl RatingRange {
    pub fn keep_above(&self, value: u64) -> RatingRange {
        RatingRange { range:
            self.range.as_ref().and_then(|r| {
                if r.end() < &value {
                    None
                } else if r.start() > &value {
                    Some(r.clone())
                } else {
                    Some(value ..= *r.end())
                }
            })
        }
    }

    pub fn keep_below(&self, value: u64) -> RatingRange {
        RatingRange { range:
            self.range.as_ref().and_then(|r| {
                if r.start() > &value {
                    None
                } else if r.end() < &value {
                    Some(r.clone())
                } else {
                    Some(*r.start() ..= value)
                }
            })
        }
    }

    pub fn len(&self) -> u64 {
        self.range.as_ref().map(|r| {
            // just in case
            assert!(r.end() >= r.start());
            // the ranges include all the valid values
            r.end() - r.start() + 1
        }).unwrap_or(0)
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug, Ord, PartialOrd)]
pub enum RatingType {
    XtremelyCool,
    Musical,
    Aerodynamic,
    Shiny,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct ConditionThresholds {
    valid_ranges: HashMap<RatingType, RatingRange>,
}

impl ConditionThresholds {
    pub fn all_allowed() -> Self {
        Self {
            valid_ranges: HashMap::from([
                (RatingType::XtremelyCool, RatingRange { range: Some(1 ..= 4000) }),
                (RatingType::Musical, RatingRange { range: Some(1 ..= 4000) }),
                (RatingType::Aerodynamic, RatingRange { range: Some(1 ..= 4000) }),
                (RatingType::Shiny, RatingRange { range: Some(1 ..= 4000) }),
            ]),
        }
    }

    pub fn keep_above(&self, rating_type: RatingType, value: u64) -> ConditionThresholds {
        let mut new_ranges = self.valid_ranges.clone();
        let old_range = self.valid_ranges.get(&rating_type).unwrap();
        new_ranges.insert(rating_type, old_range.keep_above(value));
        Self { valid_ranges: new_ranges }
    }

    pub fn keep_below(&self, rating_type: RatingType, value: u64) -> ConditionThresholds {
        let mut new_ranges = self.valid_ranges.clone();
        let old_range = self.valid_ranges.get(&rating_type).unwrap();
        new_ranges.insert(rating_type, old_range.keep_below(value));
        Self { valid_ranges: new_ranges }
    }

    pub fn number_combinations(&self) -> u64 {
        self.valid_ranges.values().map(|r| r.len()).product()
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug, Ord, PartialOrd)]
pub enum ConditionType {
    GreaterThan,
    LessThan,
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug, Ord, PartialOrd)]
pub struct PartRatings {
    xtremely_cool: u64,
    musical: u64,
    aerodynamic: u64,
    shiny: u64,
}

impl PartRatings {
    pub fn value(&self) -> u64 {
        self.xtremely_cool + self.musical + self.aerodynamic + self.shiny
    }
}

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug, Ord, PartialOrd)]
pub struct Condition {
    field: RatingType,
    threshold: u64,
    condition_type: ConditionType,
}

#[derive(PartialEq, Eq, Clone, Hash, Debug, Ord, PartialOrd)]
pub struct RuleStep {
    condition: Option<Condition>,
    target_rule: String,
}

#[derive(PartialEq, Eq, Clone, Hash, Debug, Ord, PartialOrd)]
pub struct Rule {
    name: String,
    steps: Vec<RuleStep>,
}

#[derive(PartialEq, Eq, Clone, Debug)]
pub struct Input {
    rules: HashMap<String, Rule>,
    parts: Vec<PartRatings>,
}

impl Input {
    pub fn process_part_for_rule(&self, part: PartRatings, rule: &Rule) -> bool {
        if rule.name == "A" {
            return true;
        } else if rule.name == "R" {
            return false;
        }

        for step in &rule.steps {
            match step.condition {
                Some(condition) => {
                    let relevant_value = match condition.field {
                        RatingType::XtremelyCool => part.xtremely_cool,
                        RatingType::Musical => part.musical,
                        RatingType::Aerodynamic => part.aerodynamic,
                        RatingType::Shiny => part.shiny,
                    };

                    let matched = match condition.condition_type {
                        ConditionType::GreaterThan => relevant_value > condition.threshold,
                        ConditionType::LessThan => relevant_value < condition.threshold,
                    };

                    if matched {
                        return self.process_part_for_rule(part, self.rules.get(&step.target_rule).unwrap());
                    } else {
                        continue;
                    }
                },
                None => {
                    return self.process_part_for_rule(part, self.rules.get(&step.target_rule).unwrap());
                },
            }
        }

        unreachable!();
    }

    pub fn process_part(&self, part: PartRatings) -> bool {
        self.process_part_for_rule(part, self.rules.get("in").unwrap())
    }

    pub fn invert_rule(&self, rule: &Rule) -> Vec<ConditionThresholds> {
        if rule.name == "A" {
            return vec![ConditionThresholds::all_allowed()];
        } else if rule.name == "R" {
            return vec![];
        }

        let mut overall_requirements: Vec<ConditionThresholds> = vec![];
        for step in rule.steps.iter().rev() {
            let previous_requirements = overall_requirements.clone();
            let mut new_requirements: Vec<ConditionThresholds> = vec![];
            let requirements_for_downstream = self.invert_rule(self.rules.get(&step.target_rule).unwrap());
            match step.condition {
                Some(cond) => {
                    match cond.condition_type {
                        ConditionType::GreaterThan => {
                            for req in requirements_for_downstream {
                                new_requirements.push(req.keep_above(cond.field, cond.threshold + 1));
                            }

                            for req in previous_requirements {
                                new_requirements.push(req.keep_below(cond.field, cond.threshold));
                            }
                        },
                        ConditionType::LessThan => {
                            for req in requirements_for_downstream {
                                new_requirements.push(req.keep_below(cond.field, cond.threshold - 1));
                            }

                            for req in previous_requirements {
                                new_requirements.push(req.keep_above(cond.field, cond.threshold));
                            }

                        }
                    }
                },
                None => {
                    // no condition here: we pass the overall rule now only if we pass this 
                    new_requirements = requirements_for_downstream;
                },
            }

            overall_requirements = new_requirements;
        }

        overall_requirements
    }

    pub fn get_thresholds(&self) -> Vec<ConditionThresholds> {
        self.invert_rule(self.rules.get("in").unwrap())
    }
}

pub fn part_1(input: &Input) -> u64 {
    let mut accepted_parts = Vec::new();

    for part in &input.parts {
        if input.process_part(*part) {
            accepted_parts.push(part);
        }
    }

    accepted_parts.iter().map(|p| p.value()).sum()
}

pub fn part_2(input: &Input) -> u64 {
    input.get_thresholds().iter().map(|ct| ct.number_combinations()).sum()
}

pub fn parse_rating_type(input: &str) -> IResult<&str, RatingType> {
    alt((
        map(tag("x"), |_| RatingType::XtremelyCool),
        map(tag("m"), |_| RatingType::Musical),
        map(tag("s"), |_| RatingType::Shiny),
        map(tag("a"), |_| RatingType::Aerodynamic)))(input)
}

pub fn parse_condition_type(input: &str) -> IResult<&str, ConditionType> {
    alt((
        map(tag(">"), |_| ConditionType::GreaterThan),
        map(tag("<"), |_| ConditionType::LessThan)))(input)
}

pub fn parse_condition(input: &str) -> IResult<&str, Condition> {
    let (rest, 
        (field, condition_type, threshold))
        = tuple((parse_rating_type, parse_condition_type, cc::u64))(input)?;
    Ok((rest, Condition { field, condition_type, threshold }))
}

pub fn parse_rule_step(input: &str) -> IResult<&str, RuleStep> {
    alt((
        map(
            tuple((parse_condition, tag(":"), take_while(|c: char| c.is_ascii_alphabetic()))),
                |(c, _, r)| RuleStep { condition: Some(c), target_rule: r.to_owned() },
        ),
        map(take_while(|c: char| c.is_ascii_alphabetic()), |r: &str| RuleStep { condition: None, target_rule: r.to_owned() })
    ))(input)
}

pub fn parse_rule(input: &str) -> IResult<&str, Rule> {
    let (rest, 
        (name, _, steps, _)) =
        tuple((take_while(|c: char| c.is_ascii_alphabetic()),
        tag("{"),
        separated_list1(tag(","), parse_rule_step),
        tag("}")))(input)?;

    Ok((rest, Rule { name: name.to_owned(), steps }))
}

pub fn parse_part_rating(input: &str) -> IResult<&str, PartRatings> {

let (rest, (
    _,
    xtremely_cool,
    _,
    musical,
    _,
    aerodynamic,
    _,
    shiny,
    _)) = tuple((
        tag("{x="),
        cc::u64,
        tag(",m="),
        cc::u64,
        tag(",a="),
        cc::u64,
        tag(",s="),
        cc::u64,
        tag("}")
    ))(input)?;

    Ok((rest, PartRatings { xtremely_cool, musical, aerodynamic, shiny }))
}


pub fn parse_input(input: &str) -> Input {
    let mut rules = Vec::new();
    let mut parts = Vec::new();
    let (rs, ps) = input.split_once("\n\n").unwrap();

    for line in rs.lines() {
        match all_consuming(parse_rule)(line).finish() {
            Ok((_, rule)) => rules.push(rule),
            Err(e) => { dbg!(e); unreachable!() }
        }
    }

    for line in ps.lines() {
        match all_consuming(parse_part_rating)(line).finish() {
            Ok((_, part)) => parts.push(part),
            Err(e) => { dbg!(e); unreachable!() }
        }
    }

    rules.push(Rule { name: "A".to_owned(), steps: vec![] });
    rules.push(Rule { name: "R".to_owned(), steps: vec![] });

    let rules = rules.into_iter()
        .map(|r| (r.name.clone(), r)).collect::<HashMap<_, _>>();

    Input { rules, parts }
}

fn main() {
    let input = include_str!("../input.txt");
    let input = parse_input(input);
    println!("Part 1: {}", part_1(&input));
    println!("Part 2: {}", part_2(&input));
}

#[test]
pub fn test() {
    let input = r"px{a<2006:qkq,m>2090:A,rfg}
pv{a>1716:R,A}
lnx{m>1548:A,A}
rfg{s<537:gd,x>2440:R,A}
qs{s>3448:A,lnx}
qkq{x<1416:A,crn}
crn{x>2662:A,R}
in{s<1351:px,qqz}
qqz{s>2770:qs,m<1801:hdj,R}
gd{a>3333:R,R}
hdj{m>838:A,pv}

{x=787,m=2655,a=1222,s=2876}
{x=1679,m=44,a=2067,s=496}
{x=2036,m=264,a=79,s=2244}
{x=2461,m=1339,a=466,s=291}
{x=2127,m=1623,a=2188,s=1013}";

    let input = parse_input(input);
    assert_eq!(part_1(&input), 19114);
    assert_eq!(part_2(&input), 167409079868000);
}