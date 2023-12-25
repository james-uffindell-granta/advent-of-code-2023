use std::collections::{HashSet, BTreeSet};
use itertools::Itertools;
use rand::prelude::*;
use std::time::Instant;


#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct GraphNode(String);

#[derive(Clone, PartialEq, Eq, Hash, Debug, PartialOrd, Ord)]
pub struct GraphEdge(GraphNode, GraphNode);

impl GraphEdge {
    pub fn new(vertex1: GraphNode, vertex2: GraphNode) -> GraphEdge {
        if vertex1 < vertex2 {
            GraphEdge(vertex1, vertex2)
        } else {
            GraphEdge(vertex2, vertex1)
        }
    }
}

#[derive(Clone, Debug)]
pub struct Graph {
    nodes: HashSet<GraphNode>,
    edges: HashSet<GraphEdge>,
}

impl Graph {
    pub fn to_multigraph(self) -> MultiGraph {
        MultiGraph { nodes: self.nodes.into_iter().map(|s| BTreeSet::from([s])).collect(), edges: self.edges.into_iter().collect() }
    }
}

pub fn parse_input(input: &str) -> Graph {
    let mut nodes = HashSet::new();
    let mut edges = HashSet::new();
    for line in input.lines() {
        let (node, connections) = line.split_once(": ").unwrap();
        nodes.insert(GraphNode(node.to_owned()));
        for conn in connections.split_ascii_whitespace().map(|s| s.to_owned()) {
            nodes.insert(GraphNode(conn.to_owned()));
            edges.insert(GraphEdge::new(GraphNode(node.to_owned()), GraphNode(conn.to_owned())));
        }
    }

    Graph { nodes, edges }
}

#[derive(Clone, Debug)]
pub struct MultiGraph {
    nodes: Vec<BTreeSet<GraphNode>>,
    edges: Vec<GraphEdge>,
}

impl MultiGraph {
    pub fn attempt_min_cut(mut self) -> (usize, Vec<BTreeSet<GraphNode>>) {
        while self.nodes.len() > 2 {
            let (index, random_edge) = self.edges.iter().enumerate().choose(&mut rand::thread_rng()).unwrap();
            let node_to_keep = random_edge.0.clone();
            let node_to_lose = random_edge.1.clone();
            let node_index_to_lose = self.nodes.iter().position(|s| s.contains(&node_to_lose)).unwrap();
            let nodes_lost = self.nodes.remove(node_index_to_lose);
            let node_index_to_keep = self.nodes.iter().position(|s| s.contains(&node_to_keep)).unwrap();
            self.nodes[node_index_to_keep].extend(nodes_lost);

            // self.nodes.remove(&node_to_lose);

            let new_edges_from_left = self.edges.iter()
                .filter(|e| e.0 == node_to_lose && e.1 != node_to_keep)
                .map(|e| GraphEdge::new(node_to_keep.clone(), e.1.clone()))
                .collect::<Vec<_>>();
            let new_edges_from_right = self.edges.iter()
                .filter(|e| e.1 == node_to_lose && e.0 != node_to_keep)
                .map(|e| GraphEdge::new(e.0.clone(), node_to_keep.clone()))
                .collect::<Vec<_>>();
            self.edges.retain(|e| e.1 != node_to_lose && e.0 != node_to_lose);
            self.edges.extend(new_edges_from_left);
            self.edges.extend(new_edges_from_right);
        }
        




        (self.edges.len(), self.nodes)
    }
}

pub fn part_1(graph: &Graph) -> usize {
    let graph = graph.clone().to_multigraph();
    let now = Instant::now();
    for i in 1 .. {
        if i % 100 == 0 {
            println!("Checked {}, took {:?}", i, now.elapsed());
        }
        let g = graph.clone();
        let (number_to_cut, remaining_nodes) = g.attempt_min_cut();
        if number_to_cut == 3 {
            // dbg!(remaining_nodes);
            return remaining_nodes[0].len() * remaining_nodes[1].len();
        }
    }
   
    unreachable!()
}

fn main() {
    let input = include_str!("../input.txt");
    let graph = parse_input(input);
    println!("Part 1: {}", part_1(&graph));
}

#[test]
pub fn test() {
    let input = r"jqt: rhn xhk nvd
rsh: frs pzl lsr
xhk: hfx
cmg: qnr nvd lhk bvb
rhn: xhk bvb hfx
bvb: xhk hfx
pzl: lsr hfx nvd
qnr: nvd
ntq: jqt hfx bvb xhk
nvd: lhk
lsr: lhk
rzs: qnr cmg lsr rsh
frs: qnr lhk lsr
";
    let graph = parse_input(input);
    dbg!(part_1(&graph));
}
