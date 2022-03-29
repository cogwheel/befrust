use crate::*;
use std::collections::{BTreeMap, BTreeSet};
use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};
use std::ops::Range;

pub type PinId = usize;

// TODO: make a real Pin struct
/*
#[derive(Default, Copy, Clone)]
pub struct Pin {
    id: usize,
    name: String,
}
*/

#[derive(Debug, Default, Hash, PartialEq, PartialOrd)]
struct Node {
    pin_ids: BTreeSet<usize>,
    signal: Signal,
}

impl Node {
    fn new(pin_id: PinId) -> Node {
        Node {
            pin_ids: BTreeSet::from([pin_id]),
            ..Default::default()
        }
    }
}

pub trait Part {
    fn initialize(&mut self) -> Vec<PinState>;
    fn update_pins(&mut self, input: &[PinState], output: &mut [PinState]);
}

#[derive(Default)]
pub struct Graph {
    pin_states: Vec<PinState>,
    nodes: BTreeMap<usize, Node>,
    pin_nodes: Vec<usize>, // Reverse look-up for connections

    parts: Vec<(Box<dyn Part>, Range<usize>)>,

    next_node: usize, // TODO: maybe switch back to Uuid?
}

/// Update and cycle information for a run of the graph
#[derive(Debug)]
pub struct RunStats {
    /// Number of ticks to reach steady state
    pub ticks: usize,

    /// Total number of node updates
    ///
    /// TODO: maybe there should be pins also or instead?
    pub updates: usize,

    /// Number of ticks in the final cycle
    pub cycle: usize,
}

impl Graph {
    pub fn new_pin(&mut self, state: PinState) -> PinId {
        let pin_id = self.pin_states.len();
        self.pin_states.push(state);
        let node_id = self.add_node(Node::new(pin_id));
        self.pin_nodes.push(node_id);

        assert_eq!(self.pin_states.len(), self.pin_nodes.len());

        pin_id
    }

    fn add_node(&mut self, node: Node) -> usize {
        let id = self.next_node;
        self.next_node += 1;
        let insertion = self.nodes.insert(id, node);
        assert!(matches!(insertion, None), "Uuid collision");

        id
    }

    pub fn new_const(&mut self, signal: Signal) -> PinId {
        self.new_pin(PinState::Output(signal))
    }

    pub fn new_input(&mut self) -> PinId {
        self.new_pin(PinState::Input(Signal::default()))
    }

    pub fn new_output(&mut self) -> PinId {
        self.new_pin(PinState::Output(Signal::default()))
    }

    pub fn connect(&mut self, a: PinId, b: PinId) {
        let a_node_id = self.pin_nodes[a];
        let b_node_id = self.pin_nodes[b];
        if a_node_id != b_node_id {
            // merge b into a
            let b_node = self.nodes.remove(&b_node_id).expect("Missing node");
            let a_node = self.nodes.get_mut(&a_node_id).expect("Missing_node");
            for b_pin in b_node.pin_ids.iter() {
                a_node.pin_ids.insert(b_pin.clone());
                self.pin_nodes[*b_pin] = a_node_id.clone();
            }
        } else {
            panic!("Already connected");
        }
    }

    pub fn add_part(&mut self, mut part: Box<dyn Part>) -> Vec<PinId> {
        let pin_states = part.initialize();

        let start = self.pin_states.len();
        let range = Range {
            start,
            end: start + pin_states.len(),
        };
        self.parts.push((part, range.clone()));

        for state in pin_states.iter() {
            self.new_pin(*state);
        }

        range.collect()
    }

    pub fn update_nodes(&mut self) -> usize {
        let mut update_count = 0;
        'next_node: for node in self.nodes.values_mut() {
            let mut had_output = false;
            for pin in node.pin_ids.iter() {
                match self.pin_states[*pin] {
                    PinState::HiZ | PinState::Input(_) => continue,
                    PinState::Output(signal) => {
                        if had_output {
                            node.signal = Signal::Error;
                            continue 'next_node;
                        } else {
                            had_output = true;
                            if node.signal != signal {
                                node.signal = signal;
                                update_count += 1;
                            }
                            if signal == Signal::Error {
                                continue 'next_node;
                            }
                        }
                    }
                }
            }
        }

        self.update_pins();

        update_count
    }

    fn update_pins(&mut self) {
        for (pin_id, state) in self.pin_states.iter_mut().enumerate() {
            if matches!(state, PinState::Input(_)) {
                let node_id = &self.pin_nodes[pin_id];
                let signal = self.nodes[node_id].signal;
                *state = PinState::Input(signal);
            }
        }
    }

    pub fn update_parts(&mut self) {
        let mut new_states = self.pin_states.clone();
        for (part, pin_range) in self.parts.iter_mut() {
            let start = pin_range.start;
            let end = pin_range.end;
            part.update_pins(&self.pin_states[start..end], &mut new_states[start..end]);
        }
        std::mem::swap(&mut self.pin_states, &mut new_states);
    }

    pub fn tick(&mut self) -> usize {
        self.update_parts();
        self.update_nodes()
    }

    pub fn run(&mut self) -> RunStats {
        let mut stats = RunStats {
            ticks: 0,
            updates: 0,
            cycle: 0,
        };

        // for cycle detection
        let mut state_hashes = BTreeMap::new();

        loop {
            let mut hash = DefaultHasher::new();
            self.pin_states.hash(&mut hash);
            self.nodes.hash(&mut hash);

            if let Some(tick) = state_hashes.insert(hash.finish(), stats.ticks) {
                stats.cycle = stats.ticks - tick - 1;
                break
            }

            match self.tick() {
                0 => break,
                n => {
                    stats.ticks += 1;
                    stats.updates += n;
                }
            }
        }

        stats
    }

    pub fn get_state(&self, pin_id: PinId) -> PinState {
        self.pin_states[pin_id]
    }

    pub fn get_signal(&self, pin_id: PinId) -> Signal {
        self.get_state(pin_id).into()
    }

    pub fn set_output(&mut self, pin_id: PinId, signal: Signal) {
        let state = &mut self.pin_states[pin_id];
        assert!(matches!(state, PinState::Output(_)));
        *state = PinState::Output(signal);
    }
}
