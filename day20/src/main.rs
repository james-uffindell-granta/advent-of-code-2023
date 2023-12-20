use std::collections::{HashMap, VecDeque, HashSet, BTreeMap};
use std::time::Instant;

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum PulseType {
    Low, High,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Pulse {
    sender: String,
    destination: String,
    pulse_type: PulseType,
}

impl Pulse {
    pub fn low(sender: String, destination: String) -> Self {
        Self { sender, destination, pulse_type: PulseType::Low }
    }

    pub fn high(sender: String, destination: String) -> Self {
        Self { sender, destination, pulse_type: PulseType::High }
    }

}

pub trait Module: std::fmt::Debug + std::hash::Hash {
    fn handle_pulse(&mut self, pulse: &Pulse) -> Vec<Pulse>;
    fn destinations(&self) -> Vec<String>;
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug)]
pub enum ModuleState {
    On, Off,
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct FlipFlop {
    name: String,
    destinations: Vec<String>,
    current_state: ModuleState,
}

impl FlipFlop {
    pub fn new(name: String, destinations: Vec<String>) -> Self {
        Self { name, destinations, current_state: ModuleState::Off, }
    }
}

impl Module for FlipFlop {
    fn handle_pulse(&mut self, pulse: &Pulse) -> Vec<Pulse> {
        // we could just no-op here and give every pulse to everything?
        assert!(pulse.destination == self.name);
        match pulse.pulse_type {
            PulseType::Low => {
                match self.current_state {
                    ModuleState::On => {
                        self.current_state = ModuleState::Off;
                        self.destinations.iter().map(|d| 
                            Pulse::low(self.name.clone(), d.clone())).collect()
                    },
                    ModuleState::Off => {
                        self.current_state = ModuleState::On;
                        self.destinations.iter().map(|d|
                            Pulse::high(self.name.clone(), d.clone())).collect()
                    },
                }
            },
            PulseType::High => vec![],
        }
    }

    fn destinations(&self) -> Vec<String> {
        self.destinations.clone()
    }
}

#[derive(Clone, PartialEq, Eq, Debug, Hash)]
pub struct Conjunction {
    name: String,
    destinations: Vec<String>,
    last_seen_input_pulses: BTreeMap<String, PulseType>,
}

impl Conjunction {
    pub fn new(name: String, destinations: Vec<String>, inputs: HashSet<String>) -> Self {
        Self { name, destinations, last_seen_input_pulses: inputs.into_iter().map(|i| (i, PulseType::Low)).collect() }
    }
}

impl Module for Conjunction {
    fn handle_pulse(&mut self, pulse: &Pulse) -> Vec<Pulse> {
        // we could just no-op here and give every pulse to everything?
        assert!(pulse.destination == self.name);
        self.last_seen_input_pulses.insert(pulse.sender.clone(), pulse.pulse_type);
        if self.last_seen_input_pulses.values().all(|p| p == &PulseType::High) {
            self.destinations.iter().map(|d|
                Pulse::low(self.name.clone(), d.clone())).collect()
        } else {
            self.destinations.iter().map(|d|
                Pulse::high(self.name.clone(), d.clone())).collect()
        }
    }

    fn destinations(&self) -> Vec<String> {
        self.destinations.clone()
    }
}

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Broadcaster {
    destinations: Vec<String>,
}

impl Broadcaster {
    pub fn new(destinations: Vec<String>) -> Self {
        Self { destinations, }
    }
}

impl Module for Broadcaster {
    fn handle_pulse(&mut self, pulse: &Pulse) -> Vec<Pulse> {
        self.destinations.iter().map(|d|
            Pulse { sender: String::from("broadcaster"), destination: d.clone(), pulse_type: pulse.pulse_type })
            .collect()
    }

    fn destinations(&self) -> Vec<String> {
        self.destinations.clone()
    }
}

#[derive(Debug, Hash)]
pub struct ModuleConfiguration {
    modules: BTreeMap<String, Box<dyn Module>>,
}

impl ModuleConfiguration {
    // returns (low, high)
    pub fn press_button(&mut self) -> (u64, u64, bool) {
        let (mut low_pulses, mut high_pulses) = (0_u64, 0_u64);
        let mut delivered_low_pulse_to_rx = false;
        let button_pulse = Pulse::low(String::from("button"), String::from("broadcaster"));
        let mut pulse_queue = VecDeque::new();
        pulse_queue.push_back(button_pulse);

        while let Some(pulse) = pulse_queue.pop_front() {
            match pulse.pulse_type {
                PulseType::Low => { low_pulses += 1; },
                PulseType::High => { high_pulses += 1; },
            }

            if pulse.pulse_type == PulseType::Low && pulse.destination == "rx".to_string() {
                delivered_low_pulse_to_rx = true;
            }

            // println!("Pulse {:?} being handled", pulse);
            match self.modules.get_mut(&pulse.destination) {
                Some(module) => {
                    pulse_queue.extend(module.handle_pulse(&pulse));
                },
                None => {
                    // must be an output only node?
                }
            }

            // if low_pulses + high_pulses > 50 {
            //     break;
            // }
        }

        (low_pulses, high_pulses, delivered_low_pulse_to_rx)
    }
}

pub fn parse_input(input: &str) -> ModuleConfiguration {
    let mut modules: HashMap<String, Box<dyn Module>> = HashMap::new();

    let mut conjunctions: HashMap<String, Vec<_>> = HashMap::new();
    let mut flipflops = HashMap::new();
    let mut broadcaster_outputs: Vec<String> = Vec::new();
    for line in input.lines() {
        let (sender, destinations) = line.split_once(" -> ").unwrap();
        let destinations = destinations.split(',').map(|s| s.trim().to_owned()).collect::<Vec<_>>();
        if sender.starts_with('%') {
            flipflops.insert(sender[1..].to_owned(), destinations);
        } else if sender.starts_with('&') {
            conjunctions.insert(sender[1..].to_owned(), destinations);
        } else if sender == "broadcaster" {
            // should only hit this once
            broadcaster_outputs.extend(destinations);
        }
    }

    // now we have to figure out what each conjunction's inputs are
    let mut conjunction_inputs: HashMap<String, HashSet<String>> = HashMap::new();
    for c in conjunctions.keys() {
        let conjunctions_sending_here = conjunctions.iter()
            .filter_map(|(cc, ds)| ds.contains(c).then_some(cc.clone()))
            .collect::<Vec<_>>();
        conjunction_inputs.entry(c.to_owned()).or_insert(HashSet::new()).extend(conjunctions_sending_here);
        let flipflops_sending_here = flipflops.iter()
            .filter_map(|(f, ds)| ds.contains(c).then_some(f.clone()))
            .collect::<Vec<_>>();
        conjunction_inputs.entry(c.to_owned()).or_insert(HashSet::new()).extend(flipflops_sending_here);
        if broadcaster_outputs.contains(c) {
            conjunction_inputs.entry(c.to_owned()).or_insert(HashSet::new()).insert(String::from("broadcaster"));
        }
    }

    for (f, ds) in flipflops {
        let flipflop = FlipFlop::new(f.to_owned(), ds);
        modules.insert(f.to_owned(), Box::new(flipflop));
    }

    for (c, ds) in conjunctions {
        let inputs = conjunction_inputs.get(&c).unwrap();
        let conjunction = Conjunction::new(c.to_owned(), ds, inputs.clone());
        modules.insert(c.to_owned(), Box::new(conjunction));
    }

    modules.insert(String::from("broadcaster"), Box::new(Broadcaster::new(broadcaster_outputs)));

    ModuleConfiguration { modules }
}

#[derive(Debug)]
pub struct Input {
    modules: Vec<ModuleConfiguration>,
    start_points: Vec<String>,
    destination: String,
}

pub fn parse_input_2(input: &str) -> Input {
    let mut modules: HashMap<String, Box<dyn Module>> = HashMap::new();

    let mut conjunctions: HashMap<String, Vec<_>> = HashMap::new();
    let mut flipflops = HashMap::new();
    let mut broadcaster_outputs: Vec<String> = Vec::new();
    for line in input.lines() {
        let (sender, destinations) = line.split_once(" -> ").unwrap();
        let destinations = destinations.split(',').map(|s| s.trim().to_owned()).collect::<Vec<_>>();
        if sender.starts_with('%') {
            flipflops.insert(sender[1..].to_owned(), destinations);
        } else if sender.starts_with('&') {
            conjunctions.insert(sender[1..].to_owned(), destinations);
        } else if sender == "broadcaster" {
            // should only hit this once
            broadcaster_outputs.extend(destinations);
        }
    }

    // now we have to figure out what each conjunction's inputs are
    let mut conjunction_inputs: HashMap<String, HashSet<String>> = HashMap::new();
    for c in conjunctions.keys() {
        let conjunctions_sending_here = conjunctions.iter()
            .filter_map(|(cc, ds)| ds.contains(c).then_some(cc.clone()))
            .collect::<Vec<_>>();
        conjunction_inputs.entry(c.to_owned()).or_insert(HashSet::new()).extend(conjunctions_sending_here);
        let flipflops_sending_here = flipflops.iter()
            .filter_map(|(f, ds)| ds.contains(c).then_some(f.clone()))
            .collect::<Vec<_>>();
        conjunction_inputs.entry(c.to_owned()).or_insert(HashSet::new()).extend(flipflops_sending_here);
        if broadcaster_outputs.contains(c) {
            conjunction_inputs.entry(c.to_owned()).or_insert(HashSet::new()).insert(String::from("broadcaster"));
        }
    }

    // horrible hardcoded hacky crap
    let target_node = String::from("rx");
    // assumption: there's only one conjunction that goes to rx
    let pre_target = conjunctions.iter()
            .filter_map(|(cc, ds)| ds.contains(&target_node).then_some(cc.clone()))
            .collect::<Vec<_>>().first().unwrap().clone();
    println!("Target feeding into rx is {}", pre_target);

    let mut ret = Vec::new();
    for output in broadcaster_outputs.clone() {
        println!("Processing broadcaster output {}:", output);
        let mut nodes_for_this_graph = VecDeque::new();
        let mut this_output_modules: HashMap<String, Box<dyn Module>> = HashMap::new();
        nodes_for_this_graph.push_back(output);
        while let Some(module) = nodes_for_this_graph.pop_front() {
            if this_output_modules.contains_key(&module) {
                continue;
            }

            if let Some(destinations) = flipflops.get(&module) {
                println!("Need to also consider destinations {:?}", destinations);
                nodes_for_this_graph.extend(destinations.clone());
                this_output_modules.insert(module.clone(), Box::new(FlipFlop::new(module.clone(), destinations.to_vec())));
            }

            if let Some(destinations) = conjunctions.get(&module) {
                if module == pre_target {
                    continue;
                }

                println!("Need to also consider destinations {:?}", destinations);
                nodes_for_this_graph.extend(destinations.clone());
                let inputs = conjunction_inputs.get(&module).unwrap();
                this_output_modules.insert(module.clone(), Box::new(Conjunction::new(module.clone(), destinations.to_vec(), inputs.clone())));

            }
        }

        ret.push(ModuleConfiguration { modules: this_output_modules });
    }

    Input { destination: pre_target, modules: ret, start_points: broadcaster_outputs }
}

pub fn part_1(mut module_config: ModuleConfiguration) -> u64 {
    let (mut total_low, mut total_high) = (0_u64, 0_u64);

    for _ in 1 ..= 1000 {
        let (new_low, new_high, _) = module_config.press_button();
        total_low += new_low;
        total_high += new_high;
    }

    total_low * total_high
}

pub fn part_2(input: Input) -> u64 {
    let mut seen_states = HashSet::new();
    let mut module_to_run = input.modules[0];

    let now = Instant::now();
    for button_press in 1 .. {
        let (_, _, delivered) = module_to_run.press_button();
        if button_press % 10_000 == 0 {
            println!("Pressed the button 10,000 times, time so far is {:2?}", now.elapsed());
        }
        if delivered {
            return button_press;
        }
    }

    unreachable!()
}

fn main() {
    let input = include_str!("../input.txt");
    let module_config = parse_input(input);
    println!("Part 1: {}", part_1(module_config));
    let module_config_2 = dbg!(parse_input_2(input));
    // println!("Part 2: {}", part_2(module_config_2));
}

#[test]
pub fn test() {
    let input = r"broadcaster -> a, b, c
%a -> b
%b -> c
%c -> inv
&inv -> a";

    let module_config = parse_input(input);
    assert_eq!(part_1(module_config), 32_000_000);
}


#[test]
pub fn test_loop() {
    let input = r"broadcaster -> a
%a -> inv, con
&inv -> b
%b -> con
&con -> output";

    let module_config = parse_input(input);
    assert_eq!(part_1(module_config), 11_687_500);
}