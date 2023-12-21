use std::collections::{HashMap, VecDeque, HashSet, BTreeMap};
use std::time::Instant;
use std::hash::Hash;
use num::integer::lcm;

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

#[derive(Clone, PartialEq, Eq, Hash, Debug)]
pub struct Broadcaster {
    destinations: Vec<String>,
}

impl Broadcaster {
    pub fn new(destinations: Vec<String>) -> Self {
        Self { destinations, }
    }
}

#[derive(Clone, Debug, Hash, PartialEq, Eq)]
pub enum Module {
    FlipFlop(FlipFlop),
    Conjunction(Conjunction),
    Broadcaster(Broadcaster),
}

impl Module {
    fn handle_pulse(&mut self, pulse: &Pulse) -> Vec<Pulse> {
        match self {
            Module::FlipFlop(f) => {
                // we could just no-op here and give every pulse to everything?
                assert!(pulse.destination == f.name);
                match pulse.pulse_type {
                    PulseType::Low => {
                        match f.current_state {
                            ModuleState::On => {
                                f.current_state = ModuleState::Off;
                                f.destinations.iter().map(|d| 
                                    Pulse::low(f.name.clone(), d.clone())).collect()
                            },
                            ModuleState::Off => {
                                f.current_state = ModuleState::On;
                                f.destinations.iter().map(|d|
                                    Pulse::high(f.name.clone(), d.clone())).collect()
                            },
                        }
                    },
                    PulseType::High => vec![],
                }
            },
            Module::Conjunction(c) => {
                // we could just no-op here and give every pulse to everything?
                assert!(pulse.destination == c.name);
                c.last_seen_input_pulses.insert(pulse.sender.clone(), pulse.pulse_type);
                if c.last_seen_input_pulses.values().all(|p| p == &PulseType::High) {
                    c.destinations.iter().map(|d|
                        Pulse::low(c.name.clone(), d.clone())).collect()
                } else {
                    c.destinations.iter().map(|d|
                        Pulse::high(c.name.clone(), d.clone())).collect()
                }
            },
            Module::Broadcaster(b) => {
                b.destinations.iter().map(|d|
                    Pulse { sender: String::from("broadcaster"), destination: d.clone(), pulse_type: pulse.pulse_type })
                    .collect()
            },
        }
    }

    fn destinations(&self) -> Vec<String> {
        match self {
            Module::FlipFlop(f) => f.destinations.clone(),
            Module::Conjunction(c) => c.destinations.clone(),
            Module::Broadcaster(b) => b.destinations.clone(),
        }
    }
}

#[derive(Clone, Debug, Hash, Eq, PartialEq)]
pub struct ModuleConfiguration {
    modules: BTreeMap<String, Module>,
}

impl ModuleConfiguration {
    // returns (low, high)
    pub fn press_button(&mut self) -> (u64, u64) {
        let (mut low_pulses, mut high_pulses) = (0_u64, 0_u64);
        let button_pulse = Pulse::low(String::from("button"), String::from("broadcaster"));
        let mut pulse_queue = VecDeque::new();
        pulse_queue.push_back(button_pulse);

        while let Some(pulse) = pulse_queue.pop_front() {
            match pulse.pulse_type {
                PulseType::Low => { low_pulses += 1; },
                PulseType::High => { high_pulses += 1; },
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
        }

        (low_pulses, high_pulses)
    }

    pub fn run_once(&mut self, start: &str, target: &str) -> Vec<PulseType> {
        let (mut low_pulses, mut high_pulses) = (0_u64, 0_u64);
        let mut pulses_to_target = Vec::new();
        let input_pulse = Pulse::low(String::from("broadcaster"), String::from(start));
        let mut pulse_queue = VecDeque::new();
        pulse_queue.push_back(input_pulse);

        while let Some(pulse) = pulse_queue.pop_front() {
            match pulse.pulse_type {
                PulseType::Low => { low_pulses += 1; },
                PulseType::High => { high_pulses += 1; },
            }

            if &pulse.destination == target {
                pulses_to_target.push(pulse.pulse_type);
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
        }

        pulses_to_target
    }
}

pub fn parse_input(input: &str) -> ModuleConfiguration {
    let mut modules = BTreeMap::new();

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
        modules.insert(f.to_owned(), Module::FlipFlop(flipflop));
    }

    for (c, ds) in conjunctions {
        let inputs = conjunction_inputs.get(&c).unwrap();
        let conjunction = Conjunction::new(c.to_owned(), ds, inputs.clone());
        modules.insert(c.to_owned(), Module::Conjunction(conjunction));
    }

    modules.insert(String::from("broadcaster"), Module::Broadcaster(Broadcaster::new(broadcaster_outputs)));

    ModuleConfiguration { modules }
}

#[derive(Debug)]
pub struct Input {
    modules: Vec<ModuleConfiguration>,
    start_points: Vec<String>,
    destination: String,
}

pub fn parse_input_2(input: &str) -> Input {
    let mut modules: BTreeMap<String, Module> = BTreeMap::new();

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
    // println!("Target feeding into rx is {}", pre_target);

    let mut ret = Vec::new();
    for output in broadcaster_outputs.clone() {
        // println!("Processing broadcaster output {}:", output);
        let mut nodes_for_this_graph = VecDeque::new();
        let mut this_output_modules = BTreeMap::new();
        nodes_for_this_graph.push_back(output);
        while let Some(module) = nodes_for_this_graph.pop_front() {
            if this_output_modules.contains_key(&module) {
                continue;
            }

            if let Some(destinations) = flipflops.get(&module) {
                // println!("Need to also consider destinations {:?}", destinations);
                nodes_for_this_graph.extend(destinations.clone());
                this_output_modules.insert(module.clone(), Module::FlipFlop(FlipFlop::new(module.clone(), destinations.to_vec())));
            }

            if let Some(destinations) = conjunctions.get(&module) {
                if module == pre_target {
                    continue;
                }

                // println!("Need to also consider destinations {:?}", destinations);
                nodes_for_this_graph.extend(destinations.clone());
                let inputs = conjunction_inputs.get(&module).unwrap();
                this_output_modules.insert(module.clone(), Module::Conjunction(Conjunction::new(module.clone(), destinations.to_vec(), inputs.clone())));

            }
        }

        ret.push(ModuleConfiguration { modules: this_output_modules });
    }

    Input { destination: pre_target, modules: ret, start_points: broadcaster_outputs }
}

pub fn part_1(mut module_config: ModuleConfiguration) -> u64 {
    let (mut total_low, mut total_high) = (0_u64, 0_u64);

    for _ in 1 ..= 1000 {
        let (new_low, new_high) = module_config.press_button();
        total_low += new_low;
        total_high += new_high;
    }

    total_low * total_high
}

pub fn part_2(input: &Input) -> u64 {
    let mut cycle_lengths = Vec::new();

    for (module, start) in input.modules.iter().zip(input.start_points.iter()).clone() {
        let mut seen_states = HashMap::new();
        let mut module = module.clone();
        let state = module.clone();
        seen_states.insert(state, 0);

        for button_press in 1 .. {
            let pulses_to_target = module.run_once(start, &input.destination);
            if pulses_to_target.contains(&PulseType::High) {
                println!("Sent a high pulse to target on press {}", button_press);
                println!("Pulses sent to {} this press were {:?}", input.destination, pulses_to_target);
                // there's no way we can actually know this is what we're supposed to do other than seeing
                // that this is how the cycles line up for some reason
                // and even then, I'm not convinced that we can be _sure_ it's right; what if there's a very close
                // overlap earlier on that means we luck out and still have a 'high' remembered from elsewhere?
                cycle_lengths.push(button_press as u64);
                
            } 

            let state = module.clone();
            if let Some(previous_state) = seen_states.insert(state, button_press) {
                // println!("Saw the same state after pressing the button {} times as after {} times", button_press, previous_state);
                break;
            }
        }
    }

    cycle_lengths.into_iter().reduce(lcm).unwrap()
}

fn main() {
    let input = include_str!("../input.txt");
    let module_config = parse_input(input);
    println!("Part 1: {}", part_1(module_config));
    let module_config_2 = parse_input_2(input);
    println!("Part 2: {}", part_2(&module_config_2));
    // println!("Part 2: {}", part_2_old(module_config));
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