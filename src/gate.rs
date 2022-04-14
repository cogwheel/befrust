use crate::*;
use std::iter::zip;

/// Represents a single-input, single-output logic gate
pub struct UnaryGate {
    input: Pin,
    output: Pin,
}

impl UnaryGate {
    /// Input pin
    pub fn input(&self) -> &Pin {
        &self.input
    }

    /// Output pin
    pub fn output(&self) -> &Pin {
        &self.output
    }

    /// Input pin index
    pub const INPUT: usize = 0;

    /// Output pin index
    pub const OUTPUT: usize = 1;

    /// Create a unary gate with the given updater
    pub fn new<F>(graph: &mut Graph, name: &str, updater: F) -> Self
    where
        F: 'static + FnMut(&mut [PinState]),
    {
        let pins = graph.new_part(name, &[PinState::INPUT, PinState::OUTPUT], updater);
        Self {
            input: pins[Self::INPUT].clone(),
            output: pins[Self::OUTPUT].clone(),
        }
    }
}

/// Creates a single-bit not gate
///
/// After one tick, the output will be the logical not of the input
pub fn not_gate(graph: &mut Graph, name: &str) -> UnaryGate {
    UnaryGate::new(graph, name, |pins| {
        pins[UnaryGate::OUTPUT] = PinState::Output(!pins[UnaryGate::INPUT])
    })
}

/// Creates a single-bit buffer
///
/// After one tick, the output will be the same as the input
pub fn buffer(graph: &mut Graph, name: &str) -> UnaryGate {
    UnaryGate::new(graph, name, |pins| {
        pins[UnaryGate::OUTPUT] = PinState::Output(pins[UnaryGate::INPUT].sig())
    })
}

/// Represents single-bit, two-input logic gate
pub struct BinaryGate {
    input_a: Pin,
    input_b: Pin,
    output: Pin,
}

impl BinaryGate {
    /// First input pin
    pub fn input_a(&self) -> &Pin {
        &self.input_a
    }

    /// Second input pin
    pub fn input_b(&self) -> &Pin {
        &self.input_b
    }

    /// Output pin
    pub fn output(&self) -> &Pin {
        &self.output
    }

    /// First input pin index
    pub const INPUT_A: usize = 0;

    /// Second input pin index
    pub const INPUT_B: usize = 1;

    /// Output pin index
    pub const OUTPUT: usize = 2;

    /// Creates a new BinaryGate with the given updater
    pub fn new<F>(graph: &mut Graph, name: &str, updater: F) -> Self
    where
        F: 'static + FnMut(&mut [PinState]),
    {
        let pins = graph.new_part(
            name,
            &[PinState::INPUT, PinState::INPUT, PinState::OUTPUT],
            updater,
        );
        Self {
            input_a: pins[Self::INPUT_A].clone(),
            input_b: pins[Self::INPUT_B].clone(),
            output: pins[Self::OUTPUT].clone(),
        }
    }
}

/// Create a binary and gate
///
/// After one tick, the output will be the logical and of the inputs
pub fn and_gate(graph: &mut Graph, name: &str) -> BinaryGate {
    BinaryGate::new(graph, name, |pins| {
        pins[BinaryGate::OUTPUT] =
            PinState::Output(pins[BinaryGate::INPUT_A] & pins[BinaryGate::INPUT_B]);
    })
}

/// Create a binary nand gate
///
/// After one tick, the output will be the logical nand of the inputs
pub fn nand_gate(graph: &mut Graph, name: &str) -> BinaryGate {
    BinaryGate::new(graph, name, |pins| {
        pins[BinaryGate::OUTPUT] =
            PinState::Output(!(pins[BinaryGate::INPUT_A] & pins[BinaryGate::INPUT_B]));
    })
}

/// Create a binary or gate
///
/// After one tick, the output will be the logical or of the inputs
pub fn or_gate(graph: &mut Graph, name: &str) -> BinaryGate {
    BinaryGate::new(graph, name, |pins| {
        pins[BinaryGate::OUTPUT] =
            PinState::Output(pins[BinaryGate::INPUT_A] | pins[BinaryGate::INPUT_B]);
    })
}

/// Create a binary nor gate
///
/// After one tick, the output will be the logical nor of the inputs
pub fn nor_gate(graph: &mut Graph, name: &str) -> BinaryGate {
    BinaryGate::new(graph, name, |pins| {
        pins[BinaryGate::OUTPUT] =
            PinState::Output(!(pins[BinaryGate::INPUT_A] | pins[BinaryGate::INPUT_B]));
    })
}

/// Create a binary xor gate
///
/// After one tick, the output will be the logical xor of the inputs
pub fn xor_gate(graph: &mut Graph, name: &str) -> BinaryGate {
    BinaryGate::new(graph, name, |pins| {
        pins[BinaryGate::OUTPUT] =
            PinState::Output(pins[BinaryGate::INPUT_A] ^ pins[BinaryGate::INPUT_B]);
    })
}

impl Not for &Pin {
    type Output = Pin;

    /// Creates a not gate using the given pin as its input. Returns the output pin
    fn not(self) -> Self::Output {
        let mut graph = self.graph();
        let name = format!("not({})", self.name());
        let gate = not_gate(&mut graph, &name);
        graph.connect(self, &gate.input());
        gate.output().clone()
    }
}

impl Not for Pin {
    type Output = Pin;

    /// Creates a not gate using the given pin as its input. Returns the output pin
    fn not(self) -> Self::Output {
        !&self
    }
}

impl BitAnd for &Pin {
    type Output = Pin;

    /// Creates an and gate using self and rhs as inputs. Returns the output pin
    fn bitand(self, rhs: &Pin) -> Self::Output {
        let mut graph = self.graph();
        let name = format!("and({}, {})", self.name(), rhs.name());
        let gate = and_gate(&mut graph, &name);
        graph.connect(self, gate.input_a());
        graph.connect(rhs, gate.input_b());
        gate.output().clone()
    }
}

impl BitAnd for Pin {
    type Output = Pin;

    /// Creates an and gate using self and rhs as inputs. Returns the output pin
    fn bitand(self, rhs: Pin) -> Self::Output {
        &self & &rhs
    }
}

impl BitOr for &Pin {
    type Output = Pin;

    /// Creates an or gate using self and rhs as inputs. Returns the output pin
    fn bitor(self, rhs: &Pin) -> Self::Output {
        let mut graph = self.graph();
        let name = format!("or({}, {})", self.name(), rhs.name());
        let gate = or_gate(&mut graph, &name);
        graph.connect(&self, gate.input_a());
        graph.connect(&rhs, gate.input_b());
        gate.output().clone()
    }
}

impl BitOr for Pin {
    type Output = Pin;

    /// Creates an or gate using self and rhs as inputs. Returns the output pin
    fn bitor(self, rhs: Pin) -> Self::Output {
        &self | &rhs
    }
}

impl BitXor for &Pin {
    type Output = Pin;

    /// Creates an xor gate using self and rhs as inputs. Returns the output pin
    fn bitxor(self, rhs: &Pin) -> Self::Output {
        let mut graph = self.graph();
        let name = format!("xor({}, {})", self.name(), rhs.name());
        let gate = xor_gate(&mut graph, &name);
        graph.connect(self, gate.input_a());
        graph.connect(rhs, gate.input_b());
        gate.output().clone()
    }
}

impl BitXor for Pin {
    type Output = Pin;

    /// Creates an xor gate using self and rhs as inputs. Returns the output pin
    fn bitxor(self, rhs: Pin) -> Self::Output {
        &self ^ &rhs
    }
}

/// Represents a many-input, single output logic gate
pub struct NaryGate(Vec<Pin>);

impl NaryGate {
    /// Gets the input pins for the gate
    pub fn input(&self) -> &[Pin] {
        &self.0[Self::INPUTS..]
    }

    /// Gets the nth input pin for the gate
    pub fn input_n(&self, n: usize) -> &Pin {
        &self.0[Self::INPUTS + n]
    }

    /// Gets the output pin for the gate
    pub fn output(&self) -> &Pin {
        &self.0[Self::OUTPUT]
    }

    /// Output pin index
    pub const OUTPUT: usize = 0;

    /// Input pin starting index
    pub const INPUTS: usize = 1;

    /// Creates a NaryGate with the given updater
    pub fn new<F>(graph: &mut Graph, name: &str, inputs: usize, updater: F) -> Self
    where
        F: 'static + FnMut(&mut [PinState]),
    {
        let mut states = vec![PinState::INPUT; Self::INPUTS + inputs];
        states[Self::OUTPUT] = PinState::OUTPUT;
        Self(graph.new_part(name, &states, updater))
    }

    /// Connects a sequence of pins to the inputs of the NaryGate
    pub fn connect_inputs(&self, pins: &[&Pin]) {
        for (i, pin) in pins.iter().enumerate() {
            pin.connect(self.input_n(i));
        }
    }
}

/// Creates a N-ary and gate
///
/// After one tick, the output will be the logical and of all the inputs
pub fn and_nary(graph: &mut Graph, name: &str, inputs: usize) -> NaryGate {
    NaryGate::new(graph, name, inputs, |pins| {
        let mut result = pins[NaryGate::INPUTS].sig();
        // No shortcut in case of Errors
        for state in &pins[NaryGate::INPUTS + 1..] {
            result &= state.sig();
        }

        pins[0] = PinState::Output(result);
    })
}

/// Creates a N-ary or gate
///
/// After one tick, the output will be the logical or of all the inputs
pub fn or_nary(graph: &mut Graph, name: &str, inputs: usize) -> NaryGate {
    NaryGate::new(graph, name, inputs, |pins| {
        let mut result = pins[NaryGate::INPUTS].sig();
        for state in &pins[NaryGate::INPUTS + 1..] {
            result |= state.sig();
        }

        pins[0] = PinState::Output(result);
    })
}

/// Creates a N-ary nand gate
///
/// After one tick, the output will be the inverse of the logical and of all the inputs
pub fn nand_nary(graph: &mut Graph, name: &str, inputs: usize) -> NaryGate {
    NaryGate::new(graph, name, inputs, |pins| {
        let mut result = pins[NaryGate::INPUTS].sig();
        for state in &pins[NaryGate::INPUTS + 1..] {
            result &= state.sig();
        }

        pins[0] = PinState::Output(!result);
    })
}

/// Creates a N-ary nor gate
///
/// After one tick, the output will be the inverse of the logical or of all the inputs
pub fn nor_nary(graph: &mut Graph, name: &str, inputs: usize) -> NaryGate {
    NaryGate::new(graph, name, inputs, |pins| {
        let mut result = pins[NaryGate::INPUTS].sig();
        for state in &pins[NaryGate::INPUTS + 1..] {
            result |= state.sig();
        }

        pins[0] = PinState::Output(!result);
    })
}

/// Represents a multi-bit buffer
///
/// After one tick, the buffer's output signals will match its input signals
pub struct BusBuffer(Vec<Pin>);

impl BusBuffer {
    /// Get input pins
    pub fn input(&self) -> &[Pin] {
        &self.0[self.width()..]
    }

    /// Get output pins
    pub fn output(&self) -> &[Pin] {
        &self.0[..self.width()]
    }

    /// Get the bit width of the buffer
    pub fn width(&self) -> usize {
        self.0.len() / 2
    }

    /// Create a new buffer with the given bit width
    pub fn new(graph: &mut Graph, name: &str, width: usize) -> Self {
        let mut states = vec![PinState::INPUT; 2 * width];
        states[0..width].fill(PinState::OUTPUT);

        Self(graph.new_part(name, &states, move |pins| {
            let (outs, ins) = pins.split_at_mut(width);
            for (q, a) in zip(outs, ins) {
                *q = match a {
                    PinState::HiZ => PinState::HiZ,
                    PinState::Input(sig) => PinState::Output(*sig),
                    _ => PinState::Output(Signal::Error),
                }
            }
        }))
    }
}

/// Represents a multi-bit tristate buffer
///
/// After one tick, if `en` is High, the outputs signals will match the input. Otherwise they will
/// be `HiZ`
pub struct BusTristate(Vec<Pin>);

impl BusTristate {
    /// Get the input pins
    pub fn input(&self) -> &[Pin] {
        &self.0[self.width()..2 * self.width()]
    }

    /// Get the output pins
    pub fn output(&self) -> &[Pin] {
        &self.0[..self.width()]
    }

    /// Get the enable pin
    pub fn en(&self) -> &Pin {
        &self.0.last().unwrap()
    }

    /// Get the bit width of the buffer
    pub fn width(&self) -> usize {
        (self.0.len() - 1) / 2
    }

    /// Create a new tristate buffer with the given bit width
    pub fn new(graph: &mut Graph, name: &str, width: usize) -> Self {
        let mut states = vec![PinState::INPUT; 2 * width + 1];
        // outputs start disconnected
        states[0..width].fill(PinState::HiZ);

        Self(graph.new_part(name, &states, move |pins| {
            let (outs, rest) = pins.split_at_mut(width);
            let (ins, en) = rest.split_at_mut(width);
            if en[0].is_high() {
                for (q, a) in zip(outs, ins) {
                    *q = match a {
                        PinState::HiZ => PinState::HiZ,
                        PinState::Input(sig) => PinState::Output(*sig),
                        _ => PinState::Output(Signal::Error),
                    }
                }
            } else {
                outs.fill(PinState::HiZ)
            }
        }))
    }
}

#[cfg(test)]
macro_rules! assert_sig {
    ($pin:expr, $sig:expr) => {
        assert_eq!($pin.sig(), $sig)
    };
}

#[cfg(test)]
macro_rules! assert_low {
    ($pin:expr) => {
        assert_sig!($pin, Signal::Low)
    };
}

#[cfg(test)]
macro_rules! assert_high {
    ($pin:expr) => {
        assert_sig!($pin, Signal::High)
    };
}

#[cfg(test)]
mod test_gates {
    use crate::*;

    #[test]
    fn test_not_gate() {
        let mut graph = Graph::new();

        let a = graph.new_output("a", Signal::High);
        let not1 = not_gate(&mut graph, "not2");
        let not2 = not_gate(&mut graph, "not1");

        graph.connect(&a, not1.input());
        graph.connect(not1.output(), not2.input());

        graph.run();

        assert_low!(not1.output());
        assert_high!(not2.output());
    }

    #[test]
    fn test_and_gate() {
        let mut graph = Graph::new();

        let high = graph.new_output("high", Signal::High);
        let low = graph.new_output("low", Signal::Low);
        let high2 = graph.new_output("high2", Signal::High);

        let high_and_low = and_gate(&mut graph, "and1");
        let high_and_high = and_gate(&mut graph, "and2");

        graph.connect_all(&[&high, high_and_low.input_a(), high_and_high.input_a()]);
        graph.connect(&low, high_and_low.input_b());
        graph.connect(&high2, high_and_high.input_b());

        graph.run();

        assert_low!(high_and_low.output());
        assert_high!(high_and_high.output());
    }

    #[test]
    fn test_and_nary() {
        let mut graph = Graph::new();
        let a1 = graph.new_output("a1", Signal::High);
        let a2 = graph.new_output("a2", Signal::High);
        let a3 = graph.new_output("a3", Signal::High);
        let mut a4 = graph.new_output("a4", Signal::High);

        let andy = and_nary(&mut graph, "andy", 4);

        andy.connect_inputs(&[&a1, &a2, &a3, &a4]);

        graph.run();

        assert_high!(andy.output());

        a4.set_output(Signal::Low);
        graph.run();

        assert_low!(andy.output());
    }

    #[test]
    fn test_nand_gate() {
        let mut graph = Graph::new();

        let high = graph.new_output("high", Signal::High);
        let low = graph.new_output("low", Signal::Low);
        let high2 = graph.new_output("high2", Signal::High);

        let high_nand_low = nand_gate(&mut graph, "nand1");
        let high_nand_high = nand_gate(&mut graph, "nand2");

        graph.connect_all(&[&high, high_nand_low.input_a(), high_nand_high.input_a()]);
        graph.connect(&low, high_nand_low.input_b());
        graph.connect(&high2, high_nand_high.input_b());

        graph.run();

        assert_high!(high_nand_low.output());
        assert_low!(high_nand_high.output());
    }

    #[test]
    fn test_nand_nary() {
        let mut graph = Graph::new();
        let a1 = graph.new_output("a1", Signal::High);
        let a2 = graph.new_output("a2", Signal::High);
        let a3 = graph.new_output("a3", Signal::High);
        let mut a4 = graph.new_output("a4", Signal::Low);

        let nandy = nand_nary(&mut graph, "nandy", 4);

        nandy.connect_inputs(&[&a1, &a2, &a3, &a4]);

        graph.run();

        assert_high!(nandy.output());

        a4.set_output(Signal::High);
        graph.run();

        assert_low!(nandy.output());
    }

    #[test]
    fn test_bus_buffer() {
        let mut graph = Graph::new();

        let buffer = BusBuffer::new(&mut graph, "buffer", 5);

        let mut inputs = graph.new_pins("inputs", &[PinState::Output(Signal::Low); 5]);
        for i in 0..5 {
            graph.connect(&inputs[i], &buffer.input()[i]);
        }

        graph.run();

        for i in 0..5 {
            assert_low!(buffer.output()[i]);
        }

        inputs[2].set_output(Signal::High);
        graph.run();

        for i in 0..5 {
            if i == 2 {
                assert_high!(buffer.output()[i]);
            } else {
                assert_low!(buffer.output()[i]);
            }
        }
    }

    #[test]
    fn test_bus_tristate() {
        let mut graph = Graph::new();

        let buffer = BusTristate::new(&mut graph, "buffer", 5);

        let mut inputs = graph.new_pins("inputs", &[PinState::Output(Signal::Low); 5]);
        for i in 0..5 {
            graph.connect(&inputs[i], &buffer.input()[i]);
        }

        let mut enable = graph.new_output("enable", Signal::Low);
        graph.connect(&enable, buffer.en());

        graph.run();

        for i in 0..5 {
            assert_eq!(PinState::HiZ, buffer.output()[i].state());
        }

        enable.set_output(Signal::High);
        graph.run();
        for i in 0..5 {
            assert_low!(buffer.output()[i]);
        }

        inputs[2].set_output(Signal::High);
        graph.run();

        for i in 0..5 {
            if i == 2 {
                assert_high!(buffer.output()[i]);
            } else {
                assert_low!(buffer.output()[i]);
            }
        }

        enable.set_output(Signal::Low);
        graph.run();
        for i in 0..5 {
            assert_eq!(PinState::HiZ, buffer.output()[i].state());
        }
    }
}

#[cfg(test)]
mod test_ops {
    use crate::*;

    #[test]
    fn test_unary() {
        let mut graph = Graph::new();

        let high = graph.new_output("high", Signal::High);

        let not_high = !&high;

        let yes_high = !!high;

        graph.run();

        assert_low!(not_high);
        assert_high!(yes_high);
    }

    #[test]
    fn test_binary() {
        let mut graph = Graph::new();

        let high = graph.new_output("high", Signal::High);
        let low = graph.new_output("low", Signal::Low);

        let high_and_low = &high & &low;
        let high_and_low_or_high = &high_and_low | &high;

        let high_xor_low1 = &high ^ &low;
        let high_xor_low2 = (&high & &!&low) | (&!&high & &low);

        graph.run();

        assert_low!(high_and_low);
        assert_high!(high_and_low_or_high);

        assert_high!(high_xor_low1);
        assert_high!(high_xor_low2);
    }
}
