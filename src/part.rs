use crate::*;

/*
pub struct UnaryGate {
    pub a: Pin,
    pub q: Pin,
}
*/

pub struct Gate(Vec<Pin>);

//#[derive(Gate)]  // this would implement and, or, not, etc. using q()
impl Gate {
    pub fn input(&self, i: usize) -> &Pin {
        &self.0[i + 1]
    }

    pub fn inputs(&self) -> &[Pin] {
        &self.0[1..]
    }

    pub fn output(&self) -> &Pin {
        &self.0[0]
    }

    pub fn new<F> (graph: &mut Graph, num_inputs: usize, updater: F) -> Gate
        where F: 'static + FnMut(&[PinState], &mut[PinState])
    {
        let mut pin_states = vec![PinState::INPUT; num_inputs + 1];
        pin_states[0] = PinState::OUTPUT;
        Gate(graph.add_part(&pin_states, updater))
    }

}

pub fn const_gate(graph: &mut Graph, signal: Signal) -> Gate {
    Gate(vec![graph.new_const(signal)])
}

pub fn not_gate(graph: &mut Graph) -> Gate {
    Gate::new(graph, 1, |input, output| output[0] = PinState::Output(!input[1]))
}

pub fn buffer(graph: &mut Graph) -> Gate {
    Gate::new(graph, 1, |input, output| output[0] = PinState::Output(input[1].into()))
}

pub fn and_gate(graph: &mut Graph) -> Gate {
    Gate::new(graph, 2, |input, output| output[0] = PinState::Output(input[1] & input[2]))
}

pub fn or_gate(graph: &mut Graph) -> Gate {
    Gate::new(graph, 2, |input, output| output[0] = PinState::Output(input[1] | input[2]))
}

pub fn xor_gate(graph: &mut Graph) -> Gate {
    Gate::new(graph, 2, |input, output| output[0] = PinState::Output(input[1] ^ input[2]))
}

impl Not for &Pin {
    type Output = Gate;

    fn not(self) -> Self::Output {
        let mut graph = self.graph();
        let gate = not_gate(&mut graph);
        graph.connect(&self, &gate.input(0));
        gate
    }
}

impl Not for &Gate {
    type Output = Gate;

    fn not(self) -> Self::Output {
        !self.output()
    }
}

impl BitAnd for &Pin {
    type Output = Gate;

    fn bitand(self, rhs: &Pin) -> Self::Output {
        let mut graph = self.graph();
        let gate = and_gate(&mut graph);
        graph.connect(self, gate.input(0));
        graph.connect(rhs, gate.input(1));
        gate
    }
}

impl BitAnd for &Gate {
    type Output = Gate;

    fn bitand(self, rhs: &Gate) -> Self::Output {
        self.output() & rhs.output()
    }
}

impl BitOr for &Pin {
    type Output = Gate;

    fn bitor(self, rhs: &Pin) -> Self::Output {
        let mut graph = self.graph();
        let gate = or_gate(&mut graph);
        graph.connect(self, gate.input(0));
        graph.connect(rhs, gate.input(1));
        gate
    }
}

impl BitOr for &Gate {
    type Output = Gate;

    fn bitor(self, rhs: &Gate) -> Self::Output {
        self.output() | rhs.output()
    }
}

impl BitXor for &Pin {
    type Output = Gate;

    fn bitxor(self, rhs: &Pin) -> Self::Output {
        let mut graph = self.graph();
        let gate = xor_gate(&mut graph);
        graph.connect(self, gate.input(0));
        graph.connect(rhs, gate.input(1));
        gate
    }
}

impl BitXor for &Gate {
    type Output = Gate;

    fn bitxor(self, rhs: &Gate) -> Self::Output {
        self.output() & rhs.output()
    }
}


#[cfg(test)]
mod test_not_gate {
    use crate::*;

    #[test]
    fn connect_gates() {
        let mut graph = Graph::new();
        let a = const_gate(&mut graph, Signal::High);
        let b = const_gate(&mut graph, Signal::Low);
        let a_xor_b = &a ^ &b;
        let a_xor_b2 = &(&a & &!&b) | &(&!&a & &b);
        dbg!(graph.run());
        assert_eq!(graph.get_signal(a_xor_b.output()), Signal::Low);
        assert_eq!(graph.get_signal(a_xor_b2.output()), Signal::High);
    }
}


/*
// TODO: derive constructor and part from this?
// #[derive(Part)]
pub struct RsLatch {
    //#[input]
    pub r: PinId,
    //#[input]
    pub s: PinId,

    //#[output]
    pub q: PinId,
    //#[output]
    pub q_inv: PinId,

    graph: Weak<GraphImpl>,
}

impl RsLatch {
    pub fn new(graph: &mut GraphImpl) -> RsLatch {
        let pins = graph.add_part(Box::new(RsLatchPart));
        assert_eq!(pins.len(), 4);
        if let &[r, s, q, q_inv] = &pins[0..4] {
            RsLatch { r, s, q, q_inv }
        } else {
            panic!("Unexpected pins")
        }
    }
}

struct RsLatchPart;

impl Part for RsLatchPart {
    fn initialize(&mut self) -> Vec<PinState> {
        // TODO: random initial output state?
        vec![
            PinState::INPUT,  // R
            PinState::INPUT,  // S
            PinState::OUTPUT, // Q
            PinState::OUTPUT, // NOT_Q
        ]
    }

    fn update_pins(&mut self, input: &[PinState], output: &mut [PinState]) {
        if let [PinState::Input(r), PinState::Input(s), PinState::Output(q), _] = input {
            let new_q = match (r, s) {
                (Signal::High, _) => Signal::Low,
                (Signal::Low, Signal::High) => Signal::High,
                (signal, _) | (_, signal) if !signal.is_on() => Signal::Error,
                _ => *q,
            };
            output[2] = PinState::Output(new_q);
            output[3] = PinState::Output(new_q.not());
        } else {
            panic!("Unexpected pins")
        }
    }
}

// TODO: make binary gates using truth tables
#[derive(Copy, Clone)]
pub struct AndGate {
    pub a: PinId,
    pub b: PinId,
    pub q: PinId,
}

impl AndGate {
    pub fn new(graph: &mut GraphImpl) -> AndGate {
        let pins = graph.add_part(Box::new(AndGatePart));

        AndGate {
            a: pins[0],
            b: pins[1],
            q: pins[2],
        }
    }
}

struct AndGatePart;

impl Part for AndGatePart {
    fn initialize(&mut self) -> Vec<PinState> {
        vec![PinState::INPUT, PinState::INPUT, PinState::OUTPUT]
    }

    fn update_pins(&mut self, input: &[PinState], output: &mut [PinState]) {
        if let [a, b, _] = input {
            output[2] = match (a, b) {
                (PinState::Input(a), PinState::Input(b)) => PinState::Output(match (a, b) {
                    (Signal::Error | Signal::Off, Signal::Error | Signal::Off) => Signal::Error,
                    (Signal::High, Signal::High) => Signal::High,
                    _ => Signal::Low,
                }),
                _ => panic!("Unexpected pin state"),
            };
        } else {
            panic!("Unexpected number of pins");
        }
    }
}

#[derive(Copy, Clone)]
pub struct OrGate {
    pub a: PinId,
    pub b: PinId,
    pub q: PinId,
}

impl OrGate {
    pub fn new(graph: &mut GraphImpl) -> OrGate {
        let pins = graph.add_part(Box::new(OrGatePart));

        OrGate {
            a: pins[0],
            b: pins[1],
            q: pins[2],
        }
    }
}

struct OrGatePart;

impl Part for OrGatePart {
    fn initialize(&mut self) -> Vec<PinState> {
        vec![PinState::INPUT, PinState::INPUT, PinState::OUTPUT]
    }

    fn update_pins(&mut self, input: &[PinState], output: &mut [PinState]) {
        if let [a, b, _] = input {
            output[2] = match (a, b) {
                (PinState::Input(a), PinState::Input(b)) => PinState::Output(match (a, b) {
                    (Signal::Error | Signal::Off, Signal::Error | Signal::Off) => Signal::Error,
                    (Signal::High, _) | (_, Signal::High) => Signal::High,
                    _ => Signal::Low,
                }),
                _ => panic!("Unexpected pin state"),
            };
        } else {
            panic!("Unexpected number of pins");
        }
    }
}
*/