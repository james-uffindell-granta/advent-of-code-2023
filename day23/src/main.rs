use std::{collections::{HashMap, HashSet, BTreeSet, VecDeque}, ops::Add, thread::current, cmp::Reverse};
use itertools::{Itertools, Position};
use std::time::Instant;

#[derive(PartialEq, Eq, Copy, Clone, Hash, Debug, Ord, PartialOrd)]
pub struct Coord {
    x: i64,
    y: i64,
}

impl Coord {
    pub fn neighbours(self) -> [Coord; 4] {
        [self + (0, -1), self + (0, 1), self + (-1, 0), self + (1, 0)]
    }

    pub fn next(self, direction: Direction) -> Coord {
        match direction {
            Direction::Up => self + (0, -1),
            Direction::Down => self + (0, 1),
            Direction::Left => self + (-1, 0),
            Direction::Right => self + (1, 0),
        }
    }

    pub fn previous(self, direction: Direction) -> Coord {
        match direction {
            Direction::Up => self + (0, 1),
            Direction::Down => self + (0, -1),
            Direction::Left => self + (1, 0),
            Direction::Right => self + (-1, 0),
        }
    }
}

impl From<(i64, i64)> for Coord {
    fn from((x, y): (i64, i64)) -> Self {
        Coord { x, y }
    }
}

impl Add<(i64, i64)> for Coord {
    type Output = Coord;

    fn add(self, (x, y): (i64, i64)) -> Self::Output {
        (self.x + x, self.y + y).into()
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Direction {
    Up, Down, Left, Right,
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub enum Cell {
    Path,
    Slope(Direction),
}

#[derive(Clone, Debug)]
pub struct Trails {
    cells: HashMap<Coord, Cell>,
    start_point: Coord,
    end_point: Coord,
}

impl Trails {
    pub fn to_graph(&self) -> WeightedGraph {
        let mut graph_nodes = Vec::new();
        let mut graph_edges = HashSet::new();

        // sort of BFS-style, but to find the graph nodes and their equivalent distance weights
        // the coords we've processed
        let mut visited_coords = HashSet::new();
        // map from coords we have to consider to their last intersection point
        let mut last_seen_intersections = HashMap::new();
        let mut distance_from_intersections = HashMap::new();

        let mut coords_to_consider = VecDeque::new();
        coords_to_consider.push_back(self.start_point);
        distance_from_intersections.insert(self.start_point, 0_u64);
        graph_nodes.push(GraphNode(self.start_point));

        while let Some(coord) = coords_to_consider.pop_front() {
            if visited_coords.contains(&coord) {
                // must have done this already somehow
                continue;
            }

            let neighbours = coord.neighbours().into_iter()
                .filter(|n| self.cells.contains_key(n))
                .collect::<HashSet<_>>();
            let last_seen_intersection: Option<Coord> = last_seen_intersections.get(&coord).cloned();
            let distance_here_since_last_intersection = distance_from_intersections.get(&coord).unwrap();
            if neighbours.len() < 3 {
                if coord == self.end_point {
                    // special case here - we've reached the end (should only be possible at the end?)
                    // or I guess if everything still in the queue is actually already processed
                    // (but we don't know that yet)
                    if !coords_to_consider.is_empty() {
                        // println!("Processing end point but we still need to consider {:?}", coords_to_consider);
                    }
                    assert!(coords_to_consider.iter().all(|c| visited_coords.contains(c)));
                    graph_nodes.push(GraphNode(self.end_point));
                    let last_intersection = last_seen_intersection.unwrap_or(self.start_point);
                    graph_edges.insert(WeightedGraphEdge((GraphNode(last_intersection), GraphNode(self.end_point)), *distance_here_since_last_intersection));
                    break;
                }


                let new_neighbours = neighbours.iter()
                    .filter(|&n| !visited_coords.contains(n))
                    .collect::<HashSet<_>>();

                if new_neighbours.len() != 1 {
                    // println!("Found neighbours {:?} for coord {:?}", new_neighbours, coord);
                    unreachable!();
                }

                // this is a forced route - we should have seen all but one neighbour before
                let new_neighbour = *neighbours.iter()
                    .filter(|&n| !visited_coords.contains(n))
                    .exactly_one().unwrap();
            
                // prioritize this one
                coords_to_consider.push_front(new_neighbour);
                // if we're after an intersection, remember it's the same one still
                // (if we're after the start, don't do anything, we'll interpret 'None' as the start)
                // technically new_neighbour might be an intersection itself - don't worry, we'll
                // fix that when we process it (later).
                if let Some(intersection) = last_seen_intersection {
                    last_seen_intersections.insert(new_neighbour, intersection);
                }

                distance_from_intersections.insert(new_neighbour, distance_here_since_last_intersection + 1);
            } else {
                // this is an intersection

                // let new_neighbours = neighbours.iter()
                //     .filter(|&n| !visited_coords.contains(n))
                //     .collect::<HashSet<_>>();

                // they should all be slopes - do we care about checking that
                let (inward_neighbours, outward_neighbours): (Vec<Coord>, Vec<Coord>)
                    = neighbours.into_iter()
                    .partition(|n| {
                        let pointed_to = match self.cells.get(n) {
                            Some(Cell::Slope(d)) => { n.next(*d) },
                            _ => unreachable!(),
                        };
                        pointed_to == coord
                    });
                
                // println!("At intersection {:?}, inward neighbours are {:?} and outward are {:?}", coord, inward_neighbours, outward_neighbours);

                // first a simple check: if we haven't already processed all the inward_neighbours for this
                // intersection, put it back in the queue to handle later.
                if inward_neighbours.iter().any(|n| !visited_coords.contains(n)) {
                    // println!("Haven't handled some inward neighbours yet, will revisit...");
                    coords_to_consider.push_back(coord);
                    continue;
                }

                // otherwise we've already processed all the paths that might lead here.
                // a few things we have to do.

                // firstly: 
                // remember that it is the most recent intersection itself has seen
                // this will overwrite any "wrong" beliefs in the intersections map from the other branch.
                last_seen_intersections.insert(coord, coord);
                distance_from_intersections.insert(coord, 0);

                // secondly:
                // this is a new node in our graph, so add it to the list of nodes
                graph_nodes.push(GraphNode(coord));

                // thirdly:
                // for each incoming neighbour, since we've fully handled the graph from that point,
                // we can now add this graph node and the edges into it into our list, since we know the weight there.
                // (this should also mean we're topo-sorting then graph as we build it)
                for inward in inward_neighbours {
                    let distance_so_far = distance_from_intersections.get(&inward).unwrap();
                    let distance = distance_so_far + 1;
                    let last_intersection = last_seen_intersections.get(&inward).cloned().unwrap_or(self.start_point);
                    graph_edges.insert(WeightedGraphEdge((GraphNode(last_intersection), GraphNode(coord)), distance));
                }

                // fourthly: 
                // for each outgoing neighbour, remember that we just saw an intersection and we're one away
                // and that we now need to consider these coords
                for outward in outward_neighbours {
                    last_seen_intersections.insert(outward, coord);
                    distance_from_intersections.insert(outward, 1);
                    coords_to_consider.push_front(outward);
                }
            }

            visited_coords.insert(coord);
        }


        WeightedGraph { nodes: graph_nodes, edges: graph_edges }
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct GraphNode(Coord);

#[derive(Copy, Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct WeightedGraphEdge((GraphNode, GraphNode), u64);

#[derive(Clone, Debug)]
pub struct WeightedGraph {
    nodes: Vec<GraphNode>,
    edges: HashSet<WeightedGraphEdge>,
}

impl WeightedGraph {
    pub fn get_longest_path_to_target(&self, target: GraphNode, start: GraphNode, nodes_to_ignore: &BTreeSet<GraphNode>,
            answers: &mut HashMap<(GraphNode, BTreeSet<GraphNode>), Option<u64>>) -> Option<u64> {
        if let Some(answer) =  answers.get(&(start, nodes_to_ignore.clone())) {
            return *answer;
        }
        
        if start == target {
            // already at the end - we can't leave and come back, that would enter the same square twice
            let result = Some(0);
            answers.insert((start, nodes_to_ignore.clone()), result);
            return result;
        }

        let possible_next_steps = self.edges.iter()
            .filter_map(|WeightedGraphEdge((s, e), d)| {
                if s == &start {
                    Some((*e, *d))
                } else if e == &start {
                    Some((*s, *d))
                } else {
                    None
                }
            })
            .filter(|(e, _)| !nodes_to_ignore.contains(e))
            .collect::<HashSet<_>>();

        let mut longest = None;
        let mut new_nodes_to_ignore = nodes_to_ignore.clone();
        new_nodes_to_ignore.insert(start);
        for (next_step, distance) in possible_next_steps {
            let longest_path_from_step = self.get_longest_path_to_target(target, next_step, &new_nodes_to_ignore, answers);
            let path_length = longest_path_from_step.map(|l| l + distance);
            if let Some(path) = path_length {
                match longest {
                    Some(d) if d < path => { longest = Some(path); },
                    None => { longest = Some(path); },
                    _ => { },
                };
            }
        }

        // println!("Longest path from {:?} to end ignoring {:?} is {:?} steps", start, nodes_to_ignore, longest);
        answers.insert((start, nodes_to_ignore.clone()), longest);
        longest
    }
}

pub fn parse_input(input: &str) -> Trails {
    let mut start_point = None;
    let mut end_point = None;
    let mut cells = HashMap::new();
    for (pos, (y, line)) in input.lines().enumerate().with_position() {
        for (x, c) in line.chars().enumerate() {
            let current_coord = Coord::from((x as i64, y as i64));
            match c {
                '.' => {
                    cells.insert(current_coord, Cell::Path);
                    match pos {
                        Position::First => { start_point = Some(current_coord); },
                        Position::Last => { end_point = Some(current_coord); },
                        _ => { },
                    }
                },
                '>' => { cells.insert(current_coord, Cell::Slope(Direction::Right)); },
                '<' => { cells.insert(current_coord, Cell::Slope(Direction::Left)); },
                '^' => { cells.insert(current_coord, Cell::Slope(Direction::Up)); },
                'v' => { cells.insert(current_coord, Cell::Slope(Direction::Down)); },
                _ => { },
            }
        }
    }

    let start_point = start_point.unwrap();
    let end_point = end_point.unwrap();


    Trails { cells, start_point, end_point }
}

pub fn part_1(trails: &Trails) -> u64 {
    let graph = trails.to_graph();

    let mut longest_path = HashMap::new();
    longest_path.insert(trails.start_point, 0u64);
    for g @ GraphNode(c) in graph.nodes {
        let max_path_to_me = graph.edges.iter()
            .filter(|WeightedGraphEdge((_, c2), _)| c2 == &g)
            .map(|WeightedGraphEdge((GraphNode(c1), _), d)| longest_path.get(c1).unwrap() + d)
            .max().unwrap_or(0u64);
        longest_path.insert(c, max_path_to_me);
    }


    *longest_path.get(&trails.end_point).unwrap()
}

pub fn part_2(trails: &Trails) -> u64 {
    let graph = trails.to_graph();
    let mut answers = HashMap::new();
    graph.get_longest_path_to_target(GraphNode(trails.end_point),
        GraphNode(trails.start_point),
        &BTreeSet::new(), &mut answers).unwrap()
}

fn main() {
    let input = include_str!("../input.txt");
    let trails = parse_input(input);
    println!("Part 1: {}", part_1(&trails));
    let now = Instant::now();
    println!("Part 2: {}", part_2(&trails));
    println!("Took {:2?}", now.elapsed());
}

#[test]
pub fn test() {
    let input = r"#.#####################
#.......#########...###
#######.#########.#.###
###.....#.>.>.###.#.###
###v#####.#v#.###.#.###
###.>...#.#.#.....#...#
###v###.#.#.#########.#
###...#.#.#.......#...#
#####.#.#.#######.#.###
#.....#.#.#.......#...#
#.#####.#.#.#########v#
#.#...#...#...###...>.#
#.#.#v#######v###.###v#
#...#.>.#...>.>.#.###.#
#####v#.#.###v#.#.###.#
#.....#...#...#.#.#...#
#.#########.###.#.#.###
#...###...#...#...#.###
###.###.#.###v#####v###
#...#...#.#.>.>.#.>.###
#.###.###.#.###.#.#v###
#.....###...###...#...#
#####################.#";

    let trails = parse_input(input);
    let graph = trails.to_graph();
    // dbg!(graph);
    dbg!(part_1(&trails));
    dbg!(part_2(&trails));
}