use crate::*;
use std::cell::{RefCell, RefMut};
use std::collections::hash_map::DefaultHasher;
use std::collections::{BTreeMap, BTreeSet};
use std::fmt::{Debug, Formatter};
use std::hash::{Hash, Hasher};
use std::ops::Range;
use std::rc::Rc;

pub type PinId = usize;

/// A handle to a PinState that exists in the graph
///
/// Pins are created for parts and connected together into Nodes. Signals propagate alternately from
/// nodes to parts and back, via their connected pins.
#[derive(Clone)]
pub struct Pin {
    id: PinId,
    name: String,
    graph: Graph,
}

impl Pin {
    pub fn name(&self) -> &str {
        &self.name
    }

    #[inline(always)]
    pub fn graph(&self) -> Graph {
        self.graph.clone()
    }

    #[inline(always)]
    pub fn state(&self) -> PinState {
        self.graph().get_state(self)
    }

    #[inline(always)]
    pub fn connect(&self, other: &Pin) {
        self.graph().connect(self, other);
    }

    #[inline(always)]
    pub fn connect_all(&self, others: &[&Pin]) {
        self.graph().connect_all(others);
        self.graph().connect(self, others[0]);
    }

    #[inline(always)]
    pub fn set_output(&mut self, signal: Signal) {
        self.graph().set_output(self, signal);
    }

    #[inline(always)]
    pub fn flash_output(&mut self) {
        self.graph().flash_output(self);
    }
}

impl ToSignal for Pin {
    fn sig(&self) -> Signal {
        self.graph.clone().get_signal(self)
    }
}

impl ToSignal for &Pin {
    fn sig(&self) -> Signal {
        (*self).sig()
    }
}

impl ToSignal for &&Pin {
    fn sig(&self) -> Signal {
        (**self).sig()
    }
}

impl Debug for Pin {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        let name = format!("[{}]:{}", self.id, self.name);
        f.debug_tuple(&name)
            .field(&self.graph().get_state(&self))
            .finish()
    }
}

/// A set of mutually connected pins
#[derive(Debug, Default, Hash, PartialEq, PartialOrd)]
struct Node {
    pin_ids: BTreeSet<PinId>,
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

/// A set of pins with an update function
///
/// The update function is called each tick with the latest states of the pins
type Part = Box<dyn FnMut(&mut [PinState])>;

/// The interface to the befrust compute graph
///
/// This is a shared reference so that Pins can mutate their graph for new connections
#[derive(Clone)]
pub struct Graph(Rc<RefCell<GraphImpl>>);

/// Internal data structures for the compute graph
#[derive(Default)]
struct GraphImpl {
    /// Current state of all pins in the graph
    pub pin_states: Vec<PinState>,

    /// Set of Nodes in the graph. Implemented as map for reverse lookup
    ///
    /// TODO: this could be a vector for cache friendliness but it would either:
    ///   * require extra logic to update the reverse lookup
    ///   * leave a bunch of empty nodes as pins are connected to each other
    pub nodes: BTreeMap<usize, Node>,

    /// Used to assign node ids
    pub next_node: usize,

    /// Reverse lookup for pins
    pub pin_nodes: Vec<usize>, // Reverse look-up for connections

    /// Parts for updating output pins
    pub parts: Vec<(Part, Range<usize>)>,
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

impl GraphImpl {
    fn new_pin(&mut self, state: PinState) -> PinId {
        let id = self.pin_states.len();
        self.pin_states.push(state);

        let node_id = self.next_node;
        self.next_node += 1;
        let insertion = self.nodes.insert(node_id, Node::new(id));
        assert!(matches!(insertion, None), "Node id collision");

        self.pin_nodes.push(node_id);

        assert_eq!(self.pin_states.len(), self.pin_nodes.len());

        id
    }

    fn connect(&mut self, a: &Pin, b: &Pin) {
        let a_node_id = self.pin_nodes[a.id];
        let b_node_id = self.pin_nodes[b.id];
        if a_node_id == b_node_id {
            panic!("Already connected {:?} and {:?}", a.name(), b.name());
        }
        // merge b into a
        let b_node = self.nodes.remove(&b_node_id).expect("Missing node");
        {
            let a_node = self.nodes.get_mut(&a_node_id).expect("Missing_node");
            for b_pin in b_node.pin_ids.iter() {
                a_node.pin_ids.insert(b_pin.clone());
            }
        }
        for b_pin in b_node.pin_ids.iter() {
            self.pin_nodes[*b_pin] = a_node_id.clone();
        }
    }

    pub fn update_nodes(&mut self) -> usize {
        let mut update_count = 0;
        for node in self.nodes.values_mut() {
            let mut new_signal = Signal::Off;
            let mut had_output = false;
            for pin in node.pin_ids.iter() {
                match self.pin_states[*pin] {
                    PinState::HiZ | PinState::Input(_) => continue,
                    PinState::Output(signal) => {
                        if had_output {
                            new_signal = Signal::Error;
                            break;
                        } else {
                            had_output = true;
                            new_signal = signal;
                            if signal == Signal::Error {
                                break;
                            }
                        }
                    }
                }
            }

            if new_signal != node.signal {
                update_count += 1;
                node.signal = new_signal;
                for pin_id in node.pin_ids.iter() {
                    let state = &mut self.pin_states[*pin_id];
                    if matches!(state, PinState::Input(_)) {
                        *state = PinState::Input(new_signal);
                    }
                }
            }
        }

        update_count
    }

    pub fn update_parts(&mut self) {
        for (part, pin_range) in self.parts.iter_mut() {
            let start = pin_range.start;
            let end = pin_range.end;
            part(
                &mut self.pin_states[start..end],
            );
        }
    }
}

impl Graph {
    // TODO: use IntoIter for sequence interfaces; impl IntoIter for &[&Pin]

    pub fn new() -> Self {
        Self(Rc::new(RefCell::new(GraphImpl::default())))
    }

    fn g(&self) -> RefMut<GraphImpl> {
        (*self.0).borrow_mut()
    }

    pub fn new_pin(&mut self, name: String, state: PinState) -> Pin {
        Pin {
            id: self.g().new_pin(state),
            name,
            graph: self.clone(),
        }
    }

    pub fn new_input(&mut self, name: &str) -> Pin {
        self.new_pin(name.to_owned(), PinState::Input(Signal::default()))
    }

    pub fn new_output(&mut self, name: &str, signal: Signal) -> Pin {
        self.new_pin(name.to_owned(), PinState::Output(signal))
    }

    pub fn connect(&mut self, a: &Pin, b: &Pin) {
        self.g().connect(a, b);
    }

    pub fn connect_pairs(&mut self, pairs: &[(&Pin, &Pin)]) {
        for (one, other) in pairs.iter() {
            self.connect(one, other);
        }
    }

    pub fn connect_all(&mut self, pins: &[&Pin]) {
        let (first, rest) = pins.split_first().expect("Not enough pins to connect");
        for pin in rest {
            self.g().connect(first, pin);
        }
    }

    pub fn new_pins(&mut self, name: &str, new_states: &[PinState]) -> Vec<Pin> {
        new_states
            .iter()
            .enumerate()
            .map(|(i, s)| self.new_pin(format!("{}[{}]", name, i), *s))
            .collect()
    }

    /// Creates a part
    ///
    /// A "part" is a set of pins with an associated update function. Each tick, the associated
    /// update function produces a new set of PinStates given the existing states.
    ///
    /// TODO: take (name, state) pairs
    pub fn new_part<F>(&mut self, name: &str, new_states: &[PinState], updater: F) -> Vec<Pin>
    where
        F: 'static + FnMut(&mut [PinState]),
    {
        let start = self.g().pin_states.len();
        let end = start + new_states.len();
        self.g()
            .parts
            .push((Box::new(updater), Range { start, end }));

        self.new_pins(name, new_states)
    }

    pub fn update_parts(&mut self) {
        self.g().update_parts();
    }

    pub fn update_nodes(&mut self) -> usize {
        self.g().update_nodes()
    }

    pub fn tick(&mut self) -> usize {
        self.update_parts();
        self.update_nodes()
    }

    pub fn run(&mut self) -> RunStats {
        let mut stats = RunStats {
            ticks: 1,
            updates: 0,
            cycle: 0,
        };

        // for cycle detection
        let mut state_hashes = BTreeMap::new();

        loop {
            match self.tick() {
                0 => break,
                n => {
                    stats.ticks += 1;
                    stats.updates += n;
                }
            }

            let mut hash = DefaultHasher::new();
            self.g().pin_states.hash(&mut hash);
            self.g().nodes.hash(&mut hash);

            if let Some(tick) = state_hashes.insert(hash.finish(), stats.ticks) {
                stats.cycle = stats.ticks - tick - 1;
                break;
            }
        }

        stats
    }

    pub fn get_state(&self, pin: &Pin) -> PinState {
        self.0.borrow().pin_states[pin.id]
    }

    pub fn get_signal(&self, pin: &Pin) -> Signal {
        self.get_state(pin).into()
    }

    pub fn set_output(&mut self, pin: &mut Pin, signal: Signal) {
        let state = self.g().pin_states[pin.id];
        assert!(matches!(state, PinState::Output(_)));

        self.g().pin_states[pin.id] = PinState::Output(signal);
    }

    /// Flips the state of the given output pin for one tick
    pub fn flash_output(&mut self, pin: &mut Pin) -> usize {
        let state = self.g().pin_states[pin.id];
        assert!(matches!(state, PinState::Output(_)));

        let signal = state.sig();
        self.g().pin_states[pin.id] = PinState::Output(!signal);
        let updates = self.tick();
        self.g().pin_states[pin.id] = PinState::Output(signal);

        updates
    }
}

#[cfg(test)]
mod test_graph {
    use crate::*;

    #[test]
    fn create_graph() {
        let mut graph = Graph::new();

        let a = graph.new_output("a", Signal::High);
        //let b = graph.new_output(Signal::High);

        let pins = graph.new_part(
            "not_gate",
            &[PinState::INPUT, PinState::OUTPUT],
            |pins| {
                pins[1] = PinState::Output(!(pins[0]));
            },
        );

        graph.connect(&a, &pins[0]);

        graph.run();

        assert_eq!(graph.get_signal(&pins[1]), Signal::Low);
    }
}
