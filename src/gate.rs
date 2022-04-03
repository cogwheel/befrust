use crate::*;

use derive_getters::Getters;

#[derive(Getters)]
pub struct UnaryGate {
    a: Pin,
    q: Pin,
}

impl UnaryGate {
    pub const A: usize = 0;
    pub const Q: usize = 1;

    pub fn new<F>(graph: &mut Graph, name: &str, updater: F) -> UnaryGate
    where
        F: 'static + Fn(&[PinState], &mut [PinState]),
    {
        let pins = graph.new_part(name, &[PinState::INPUT, PinState::OUTPUT], updater);
        UnaryGate {
            a: pins[Self::A].clone(),
            q: pins[Self::Q].clone(),
        }
    }
}

pub fn not_gate(graph: &mut Graph, name: &str) -> UnaryGate {
    UnaryGate::new(graph, name, |before, after| {
        after[UnaryGate::Q] = PinState::Output(!before[UnaryGate::A])
    })
}

pub fn buffer(graph: &mut Graph, name: &str) -> UnaryGate {
    UnaryGate::new(graph, name, |before, after| {
        after[UnaryGate::Q] = before[UnaryGate::A]
    })
}

#[derive(Getters)]
pub struct BinaryGate {
    a: Pin,
    b: Pin,
    q: Pin,
}

impl BinaryGate {
    pub const A: usize = 0;
    pub const B: usize = 1;
    pub const Q: usize = 2;

    pub fn new<F>(graph: &mut Graph, name: &str, updater: F) -> BinaryGate
    where
        F: 'static + Fn(&[PinState], &mut [PinState]),
    {
        let pins = graph.new_part(
            name,
            &[PinState::INPUT, PinState::INPUT, PinState::OUTPUT],
            updater,
        );
        BinaryGate {
            a: pins[Self::A].clone(),
            b: pins[Self::B].clone(),
            q: pins[Self::Q].clone(),
        }
    }
}

pub fn and_gate(graph: &mut Graph, name: &str) -> BinaryGate {
    BinaryGate::new(graph, name, |before, after| {
        after[BinaryGate::Q] = PinState::Output(before[BinaryGate::A] & before[BinaryGate::B]);
    })
}

pub fn or_gate(graph: &mut Graph, name: &str) -> BinaryGate {
    BinaryGate::new(graph, name, |before, after| {
        after[BinaryGate::Q] = PinState::Output(before[BinaryGate::A] | before[BinaryGate::B]);
    })
}

pub fn xor_gate(graph: &mut Graph, name: &str) -> BinaryGate {
    BinaryGate::new(graph, name, |before, after| {
        after[BinaryGate::Q] = PinState::Output(before[BinaryGate::A] ^ before[BinaryGate::B]);
    })
}

impl Not for &Pin {
    type Output = Pin;

    fn not(self) -> Self::Output {
        let mut graph = self.graph();
        let name = format!("not({})", self.name());
        let gate = not_gate(&mut graph, &name);
        graph.connect(self, &gate.a());
        gate.q().clone()
    }
}

impl Not for Pin {
    type Output = Pin;

    fn not(self) -> Self::Output {
        !&self
    }
}

impl BitAnd for &Pin {
    type Output = Pin;

    fn bitand(self, rhs: &Pin) -> Self::Output {
        let mut graph = self.graph();
        let name = format!("and({}, {})", self.name(), rhs.name());
        let gate = and_gate(&mut graph, &name);
        graph.connect(self, gate.a());
        graph.connect(rhs, gate.b());
        gate.q().clone()
    }
}

impl BitAnd for Pin {
    type Output = Pin;

    fn bitand(self, rhs: Pin) -> Self::Output {
        &self & &rhs
    }
}

impl BitOr for &Pin {
    type Output = Pin;

    fn bitor(self, rhs: &Pin) -> Self::Output {
        let mut graph = self.graph();
        let name = format!("or({}, {})", self.name(), rhs.name());
        let gate = or_gate(&mut graph, &name);
        graph.connect(&self, gate.a());
        graph.connect(&rhs, gate.b());
        gate.q().clone()
    }
}

impl BitOr for Pin {
    type Output = Pin;

    fn bitor(self, rhs: Pin) -> Self::Output {
        &self | &rhs
    }
}

impl BitXor for &Pin {
    type Output = Pin;

    fn bitxor(self, rhs: &Pin) -> Self::Output {
        let mut graph = self.graph();
        let name = format!("xor({}, {})", self.name(), rhs.name());
        let gate = xor_gate(&mut graph, &name);
        graph.connect(self, gate.a());
        graph.connect(rhs, gate.b());
        gate.q().clone()
    }
}

impl BitXor for Pin {
    type Output = Pin;

    fn bitxor(self, rhs: Pin) -> Self::Output {
        &self ^ &rhs
    }
}

pub struct NaryGate(Vec<Pin>);

impl NaryGate {
    pub fn a(&self) -> &[Pin] {
        &self.0[1..]
    }
    pub fn n(&self, n: usize) -> &Pin {
        &self.0[n + 1]
    }
    pub fn q(&self) -> &Pin {
        &self.0[0]
    }

    pub fn new<F>(graph: &mut Graph, name: &str, inputs: usize, updater: F) -> NaryGate
    where
        F: 'static + Fn(&[PinState], &mut [PinState]),
    {
        let mut states = vec![PinState::INPUT; inputs + 1];
        states[0] = PinState::OUTPUT;
        NaryGate(graph.new_part(name, &states, updater))
    }

    pub fn connect_inputs(&self, pins: &[&Pin]) {
        for (i, pin) in pins.iter().enumerate() {
            pin.connect(self.n(i));
        }
    }
}

pub fn and_nary(graph: &mut Graph, name: &str, inputs: usize) -> NaryGate {
    NaryGate::new(graph, name, inputs, |before, after| {
        let mut result = before[1].sig();
        for state in &before[2..] {
            result &= state.sig();
        }

        after[0] = PinState::Output(result);
    })
}

pub fn nand_nary(graph: &mut Graph, name: &str, inputs: usize) -> NaryGate {
    NaryGate::new(graph, name, inputs, |before, after| {
        let mut result = before[1].sig();
        for state in &before[2..] {
            result &= state.sig();
        }

        after[0] = PinState::Output(!result);
    })
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

        graph.connect(&a, not1.a());
        graph.connect(not1.q(), not2.a());

        graph.run();

        assert_low!(not1.q());
        assert_high!(not2.q());
    }

    #[test]
    fn test_and_gate() {
        let mut graph = Graph::new();

        let high = graph.new_output("a", Signal::High);
        let low = graph.new_output("a", Signal::Low);
        let high2 = graph.new_output("a", Signal::High);

        let high_and_low = and_gate(&mut graph, "and1");
        let high_and_high = and_gate(&mut graph, "and2");

        graph.connect_all(&[&high, high_and_low.a(), high_and_high.a()]);
        graph.connect(&low, high_and_low.b());
        graph.connect(&high2, high_and_high.b());

        graph.run();

        assert_low!(high_and_low.q());
        assert_high!(high_and_high.q());
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

        assert_high!(andy.q());

        a4.set_output(Signal::Low);
        graph.run();

        assert_low!(andy.q());
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

        assert_high!(nandy.q());

        a4.set_output(Signal::High);
        graph.run();

        assert_low!(nandy.q());
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