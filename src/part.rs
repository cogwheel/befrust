use crate::*;

pub struct NotGate {
    pub a: PinId,
    pub q: PinId,
}

impl NotGate {
    pub fn new(graph: &mut Graph) -> NotGate {
        let pin_ids = graph.add_part(Box::new(NotGatePart));
        NotGate {
            a: pin_ids[0],
            q: pin_ids[1],
        }
    }
}

struct NotGatePart;

impl Part for NotGatePart {
    fn initialize(&mut self) -> Vec<PinState> {
        vec![PinState::INPUT, PinState::OUTPUT]
    }

    fn update_pins(&mut self, input: &[PinState], output: &mut [PinState]) {
        output[1] = match input[0] {
            PinState::Input(signal) => PinState::Output(signal.not()),
            _ => panic!("Unexpected pin state"),
        }
    }
}

// TODO: derive constructor and part from this
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
}

impl RsLatch {
    pub fn new(graph: &mut Graph) -> RsLatch {
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
    pub fn new(graph: &mut Graph) -> AndGate {
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
    pub fn new(graph: &mut Graph) -> OrGate {
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
