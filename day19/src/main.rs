
use nom::{
    branch::alt,
    bytes::complete::{tag, take_while},
    character::is_alphabetic,
    character::complete as cc,
    combinator::{all_consuming, map},
    multi::separated_list1,
    sequence::{separated_pair, tuple},
    Finish,
    IResult,
};

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug, Ord, PartialOrd)]
pub enum RatingType {
    XtremelyCool,
    Musical,
    Aerodynamic,
    Shiny,
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
    conditions: Vec<RuleStep>,
}

#[derive(PartialEq, Eq, Clone, Hash, Debug, Ord, PartialOrd)]
pub struct Input {
    rules: Vec<Rule>,
    parts: Vec<PartRatings>,
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

    Ok((rest, Rule { name: name.to_owned(), conditions: steps }))
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

    Input { rules, parts }
}



fn main() {
    println!("Hello, world!");
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
    dbg!(input);
}
