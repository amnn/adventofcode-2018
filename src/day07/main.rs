#[macro_use] extern crate lib;
#[macro_use] extern crate scan_fmt;

use std::cmp::{Ord, PartialOrd, Ordering};
use std::collections::{BinaryHeap, BTreeMap, BTreeSet};
use std::env;
use std::fs::File;
use std::io::{self, BufRead, BufReader};

input! {
    #["Step {} must be finished before step {}"; "can begin."]
    struct Dep {
        before: char,
        after: char
    }
}

struct Node {
    in_edges: BTreeSet<char>,
    out_edges: BTreeSet<char>,
}

impl Node {
    fn new() -> Node {
        Node { in_edges: BTreeSet::new(), out_edges: BTreeSet::new() }
    }
}

struct Graph {
    nodes: BTreeMap<char, Node>,
    available: BTreeSet<char>,
    reserved: BTreeSet<char>,
}

impl Graph {
    fn new() -> Graph {
        Graph {
            nodes: BTreeMap::new(),
            available: BTreeSet::new(),
            reserved: BTreeSet::new()
        }
    }

    fn node_for<'a>(&'a mut self, id: char) -> &'a mut Node {
        let Graph { nodes, available, .. } = self;
        nodes.entry(id).or_insert_with( || {
            available.insert(id);
            Node::new()
        })
    }

    fn add_edge(&mut self, from: char, to: char) {
        self.node_for(from).out_edges.insert(to);
        self.node_for(to).in_edges.insert(from);

        self.available.remove(&to);
    }

    fn remove_node(&mut self, id: char) {
        let Graph { nodes, available, reserved } = self;
        let removed = nodes.remove(&id).unwrap();
        reserved.remove(&id);

        for out in removed.out_edges {
            let mut nbr = nodes.get_mut(&out).unwrap();
            nbr.in_edges.remove(&id);

            if nbr.in_edges.is_empty() {
                available.insert(out);
            }
        }
    }

    fn first_available(&self) -> char {
        *self.available.iter().next().unwrap()
    }

    fn reserve(&mut self, id: char) {
        self.available.remove(&id);
        self.reserved.insert(id);
    }

    fn has_available(&self) -> bool {
        !self.available.is_empty()
    }
}

#[derive(Eq, PartialEq)]
struct WorkItem {
    id: char,
    fin: usize,
}

impl Ord for WorkItem {
    fn cmp(&self, other: &WorkItem) -> Ordering {
        other.fin.cmp(&self.fin)
            .then_with(|| self.id.cmp(&other.id))
    }
}

impl PartialOrd for WorkItem {
    fn partial_cmp(&self, other: &WorkItem) -> Option<Ordering> {
        Some(other.fin.cmp(&self.fin))
    }
}

fn cost(id: char) -> usize {
    id as usize - 'A' as usize + 61
}

fn main() -> io::Result<()> {
    let fname = env::args().nth(1).unwrap();
    let file = File::open(fname)?;
    let reader = BufReader::new(file);

    let deps = reader
        .lines()
        .map(|l| Dep::new(&l?))
        .collect::<io::Result<Vec<_>>>()?;

    {
        let mut graph = Graph::new();
        for dep in &deps {
            graph.add_edge(dep.before, dep.after);
        }

        let mut order = String::new();
        while graph.has_available() {
            let next = graph.first_available();
            graph.reserve(next);
            graph.remove_node(next);
            order.push(next);
        }

        println!("Part 1: {}", order);
    }

    {
        let mut graph = Graph::new();
        // Insert a sentinel node.
        graph.node_for('\0');
        for dep in &deps {
            graph.add_edge(dep.before, dep.after);
        }

        let mut pending = BinaryHeap::new();

        // Pretend like we finished working on the sentinal node to start
        // things off.
        graph.reserve('\0');
        pending.push(WorkItem { id: '\0', fin: 0 });

        // One fewer worker than we have because one worker is assigned to the
        // sentinel work item.
        let mut workers = 4;
        let mut time = 0;

        while !pending.is_empty() {
            let WorkItem { id, fin } = pending.pop().unwrap();
            graph.remove_node(id);
            workers += 1;
            time = fin;

            // Assign work to available workers.
            while workers > 0 && graph.has_available() {
                let next = graph.first_available();
                graph.reserve(next);
                pending.push(WorkItem { id: next, fin: time + cost(next) });
                workers -= 1;
            }
        }

        println!("Part 2: {}", time);
    }

    Ok(())
}
