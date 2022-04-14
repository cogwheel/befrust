use crate::*;
use derive_getters::Getters;

#[derive(Debug, Getters)]
pub struct TFlipFlop {
    toggle: Pin,
    set: Pin,
    reset: Pin,
    output: Pin,
    out_inv: Pin,
}

impl TFlipFlop {
    // external
    const TOGGLE: usize = 0;
    const SET: usize = 1;
    const RESET: usize = 2;
    const OUTPUT: usize = 3;
    const OUT_INV: usize = 4;

    // internal for edge detection
    const TOGGLE_PREV: usize = 5;

    pub fn new(graph: &mut Graph, name: &str) -> Self {
        let pins = graph.new_part(
            name,
            &[
                PinState::INPUT,
                PinState::INPUT,
                PinState::INPUT,
                PinState::Output(Signal::Low),
                PinState::Output(Signal::High),
                PinState::INPUT,
            ],
            |pins| {
                let new_q = if pins[Self::RESET].is_high() {
                    Signal::Low
                } else if pins[Self::SET].is_high() {
                    Signal::High
                } else if pins[Self::TOGGLE].is_high() && pins[Self::TOGGLE_PREV].is_lowish() {
                    pins[Self::OUT_INV].sig()
                } else {
                    pins[Self::OUTPUT].sig()
                };
                pins[Self::OUTPUT] = PinState::Output(new_q);
                pins[Self::OUT_INV] = PinState::Output(!new_q);
                pins[Self::TOGGLE_PREV] = pins[Self::TOGGLE];
            },
        );
        Self {
            toggle: pins[Self::TOGGLE].clone(),
            set: pins[Self::SET].clone(),
            reset: pins[Self::RESET].clone(),
            output: pins[Self::OUTPUT].clone(),
            out_inv: pins[Self::OUT_INV].clone(),
        }
    }
}

#[derive(Debug, Getters)]
pub struct HalfAdder {
    input: Pin,
    clear: Pin,
    load: Pin,

    #[getter(skip)]
    flip_flop: TFlipFlop,
}

impl HalfAdder {
    pub fn toggle(&self) -> &Pin {
        self.flip_flop.toggle()
    }
    pub fn output(&self) -> &Pin {
        self.flip_flop.output()
    }
    pub fn out_inv(&self) -> &Pin {
        self.flip_flop.out_inv()
    }

    pub fn new(graph: &mut Graph, name: &str) -> Self {
        let input = graph.new_input(&format!("{}.input", name));
        let clear = graph.new_input(&format!("{}.clear", name));
        let load = graph.new_input(&format!("{}.load", name));
        let flip_flop = TFlipFlop::new(graph, &format!("{}.flip_flop", name));

        let use_input = !(&input & &load & clear.clone());
        let set = !&use_input;

        let load_ff = !(&use_input & &load);
        let reset_ff = !&clear | !load_ff;

        graph.connect(&set, flip_flop.set());
        graph.connect(&reset_ff, flip_flop.reset());

        Self {
            input,
            clear,
            load,
            flip_flop,
        }
    }
}

#[derive(Debug, Getters)]
pub struct FullAdder {
    up: Pin,
    down: Pin,
    up_cond: Pin,
    down_cond: Pin,

    #[getter(skip)]
    half_adder: HalfAdder,
}

impl FullAdder {
    pub fn input(&self) -> &Pin {
        &self.half_adder.input()
    }
    pub fn clear(&self) -> &Pin {
        self.half_adder.clear()
    }
    pub fn load(&self) -> &Pin {
        self.half_adder.load()
    }
    pub fn output(&self) -> &Pin {
        self.half_adder.output()
    }
    pub fn out_inv(&self) -> &Pin {
        self.half_adder.out_inv()
    }

    pub fn new(graph: &mut Graph, name: &str) -> Self {
        let adder = Self {
            up: graph.new_input(&format!("{}.up", name)),
            down: graph.new_input(&format!("{}.down", name)),
            up_cond: graph.new_input(&format!("{}.up_cond", name)),
            down_cond: graph.new_input(&format!("{}.down_cond", name)),
            half_adder: HalfAdder::new(graph, &format!("{}.half", name)),
        };

        let toggle = !(adder.up() & adder.up_cond() | adder.down() & adder.down_cond());

        graph.connect(&toggle, adder.half_adder.toggle());

        adder
    }
}

#[derive(Getters)]
pub struct Ic74193 {
    // inputs
    up: Pin,
    down: Pin,
    load_inv: Pin,
    clear: Pin,

    // outputs
    borrow: Pin,
    carry: Pin,

    #[getter(skip)]
    adder1: HalfAdder,
    #[getter(skip)]
    adder2: FullAdder,
    #[getter(skip)]
    adder3: FullAdder,
    #[getter(skip)]
    adder4: FullAdder,
}

impl Ic74193 {
    pub fn input(&self) -> [&Pin; 4] {
        [self.in1(), self.in2(), self.in3(), self.in4()]
    }
    pub fn in1(&self) -> &Pin {
        self.adder1.input()
    }
    pub fn in2(&self) -> &Pin {
        self.adder2.input()
    }
    pub fn in3(&self) -> &Pin {
        self.adder3.input()
    }
    pub fn in4(&self) -> &Pin {
        self.adder4.input()
    }

    pub fn output(&self) -> [&Pin; 4] {
        [self.out1(), self.out2(), self.out3(), self.out4()]
    }
    pub fn out1(&self) -> &Pin {
        self.adder1.output()
    }
    pub fn out2(&self) -> &Pin {
        self.adder2.output()
    }
    pub fn out3(&self) -> &Pin {
        self.adder3.output()
    }
    pub fn out4(&self) -> &Pin {
        self.adder4.output()
    }

    pub fn new(graph: &mut Graph, name: &str) -> Self {
        let make_name = |part| format!("{}.{}", name, part);
        let up_inv = not_gate(graph, &make_name("up_inv"));
        let down_inv = not_gate(graph, &make_name("down_inv"));
        let load = not_gate(graph, &make_name("load"));
        let clear_inv = not_gate(graph, &make_name("clear_inv"));
        let adder1 = HalfAdder::new(graph, &make_name("adder1"));
        let adder2 = FullAdder::new(graph, &make_name("adder2"));
        let adder3 = FullAdder::new(graph, &make_name("adder3"));
        let adder4 = FullAdder::new(graph, &make_name("adder4"));

        let carry = nand_nary(graph, &make_name("carry"), 5);
        carry.connect_inputs(&[
            up_inv.output(),
            adder1.output(),
            adder2.output(),
            adder3.output(),
            adder4.output(),
        ]);

        let borrow = nand_nary(graph, &make_name("borrow"), 5);
        borrow.connect_inputs(&[
            down_inv.output(),
            adder1.out_inv(),
            adder2.out_inv(),
            adder3.out_inv(),
            adder4.out_inv(),
        ]);

        let toggle1 = !(up_inv.output() | down_inv.output());

        toggle1.connect(adder1.toggle());

        let up_cond2 = adder1.output();
        let down_cond2 = adder1.out_inv();
        graph.connect(&up_cond2, adder2.up_cond());
        graph.connect(&down_cond2, adder2.down_cond());

        let up_cond3 = up_cond2 & adder2.output();
        let down_cond3 = down_cond2 & adder2.out_inv();
        graph.connect(&up_cond3, adder3.up_cond());
        graph.connect(&down_cond3, adder3.down_cond());

        let up_cond4 = &up_cond3 & adder3.output();
        let down_cond4 = &down_cond3 & adder3.out_inv();
        graph.connect(&up_cond4, adder4.up_cond());
        graph.connect(&down_cond4, adder4.down_cond());

        up_inv
            .output()
            .connect_all(&[adder2.up(), adder3.up(), adder4.up()]);

        down_inv
            .output()
            .connect_all(&[adder2.down(), adder3.down(), adder4.down()]);

        load.output()
            .connect_all(&[adder1.load(), adder2.load(), adder3.load(), adder4.load()]);

        clear_inv.output().connect_all(&[
            adder1.clear(),
            adder2.clear(),
            adder3.clear(),
            adder4.clear(),
        ]);

        Self {
            up: up_inv.input().clone(),
            down: down_inv.input().clone(),
            load_inv: load.input().clone(),
            clear: clear_inv.input().clone(),
            carry: carry.output().clone(),
            borrow: borrow.output().clone(),
            adder1,
            adder2,
            adder3,
            adder4,
        }
    }
}

pub struct Counter8Bit {
    counter1: Ic74193,
    counter2: Ic74193,
}

impl Counter8Bit {
    pub fn input(&self) -> [&Pin; 8] {
        [
            self.counter1.in1(),
            self.counter1.in2(),
            self.counter1.in3(),
            self.counter1.in4(),
            self.counter2.in1(),
            self.counter2.in2(),
            self.counter2.in3(),
            self.counter2.in4(),
        ]
    }

    pub fn up(&self) -> &Pin {
        self.counter1.up()
    }

    pub fn down(&self) -> &Pin {
        self.counter1.down()
    }

    pub fn load_inv(&self) -> &Pin {
        self.counter1.load_inv()
    }

    pub fn clear(&self) -> &Pin {
        self.counter1.clear()
    }

    pub fn output(&self) -> [&Pin; 8] {
        [
            self.counter1.out1(),
            self.counter1.out2(),
            self.counter1.out3(),
            self.counter1.out4(),
            self.counter2.out1(),
            self.counter2.out2(),
            self.counter2.out3(),
            self.counter2.out4(),
        ]
    }

    pub fn carry(&self) -> &Pin {
        self.counter2.carry()
    }

    pub fn borrow(&self) -> &Pin {
        self.counter2.borrow()
    }

    pub fn new(graph: &mut Graph, name: &str) -> Self {
        let counter1 = Ic74193::new(graph, &format!("{}.counter1", name));
        let counter2 = Ic74193::new(graph, &format!("{}.counter2", name));

        graph.connect_pairs(&[
            (counter1.carry(), counter2.up()),
            (counter1.borrow(), counter2.down()),
            (counter1.load_inv(), counter2.load_inv()),
            (counter1.clear(), counter2.clear()),
        ]);

        Self { counter1, counter2 }
    }
}

/// A 16-bit counter built from Ic74193s
///
/// Note: this would be a lot more efficient as a straight up part with its own update function to
/// do the math outside the simulation, but that kind of misses the point ;)
pub struct Counter16Bit {
    counter1: Counter8Bit,
    counter2: Counter8Bit,
}

impl Counter16Bit {
    pub fn d(&self) -> [&Pin; 16] {
        [
            &self.counter1.input()[0],
            &self.counter1.input()[1],
            &self.counter1.input()[2],
            &self.counter1.input()[3],
            &self.counter1.input()[4],
            &self.counter1.input()[5],
            &self.counter1.input()[6],
            &self.counter1.input()[7],
            &self.counter2.input()[0],
            &self.counter2.input()[1],
            &self.counter2.input()[2],
            &self.counter2.input()[3],
            &self.counter2.input()[4],
            &self.counter2.input()[5],
            &self.counter2.input()[6],
            &self.counter2.input()[7],
        ]
    }

    pub fn up(&self) -> &Pin {
        self.counter1.up()
    }

    pub fn down(&self) -> &Pin {
        self.counter1.down()
    }

    pub fn load_inv(&self) -> &Pin {
        self.counter1.load_inv()
    }

    pub fn clear(&self) -> &Pin {
        self.counter1.clear()
    }

    pub fn output(&self) -> [&Pin; 16] {
        [
            &self.counter1.output()[0],
            &self.counter1.output()[1],
            &self.counter1.output()[2],
            &self.counter1.output()[3],
            &self.counter1.output()[4],
            &self.counter1.output()[5],
            &self.counter1.output()[6],
            &self.counter1.output()[7],
            &self.counter2.output()[0],
            &self.counter2.output()[1],
            &self.counter2.output()[2],
            &self.counter2.output()[3],
            &self.counter2.output()[4],
            &self.counter2.output()[5],
            &self.counter2.output()[6],
            &self.counter2.output()[7],
        ]
    }

    pub fn carry(&self) -> &Pin {
        self.counter2.carry()
    }

    pub fn borrow(&self) -> &Pin {
        self.counter2.borrow()
    }

    pub fn new(graph: &mut Graph, name: &str) -> Self {
        let counter1 = Counter8Bit::new(graph, &format!("{}.counter1", name));
        let counter2 = Counter8Bit::new(graph, &format!("{}.counter2", name));

        graph.connect_pairs(&[
            (counter1.carry(), counter2.up()),
            (counter1.borrow(), counter2.down()),
            (counter1.load_inv(), counter2.load_inv()),
            (counter1.clear(), counter2.clear()),
        ]);

        Self { counter1, counter2 }
    }
}

pub struct IcCY7C199(Vec<Pin>);

impl IcCY7C199 {
    const CE_INV: usize = 0;
    const OE_INV: usize = 1;
    const WE_INV: usize = 2;

    const IO_START: usize = 3;
    const WORD_SIZE: usize = 8;
    const IO_END: usize = Self::IO_START + Self::WORD_SIZE;

    const ADDR_START: usize = Self::IO_END;
    const ADDR_SIZE: usize = 15;
    const ADDR_END: usize = Self::ADDR_START + Self::ADDR_SIZE;

    const NUM_PINS: usize = Self::ADDR_END;

    const NUM_WORDS: usize = 1 << Self::ADDR_SIZE;

    pub fn ce_inv(&self) -> &Pin {
        &self.0[Self::CE_INV]
    }

    pub fn oe_inv(&self) -> &Pin {
        &self.0[Self::OE_INV]
    }

    pub fn we_inv(&self) -> &Pin {
        &self.0[Self::WE_INV]
    }

    pub fn io(&self) -> &[Pin] {
        &self.0[Self::IO_START..Self::IO_END]
    }

    pub fn addr(&self) -> &[Pin] {
        &self.0[Self::ADDR_START..Self::ADDR_END]
    }

    pub fn new(graph: &mut Graph, name: &str) -> Self {
        let mut states = [PinState::INPUT; Self::NUM_PINS];
        Self::set_io(&mut states, PinState::HiZ);

        let mut ram = vec![0xff; Self::NUM_WORDS];

        let pins = graph.new_part(name, &states, move |pins| {
            Self::update(&mut ram, pins);
        });

        Self(pins)
    }

    fn set_io(states: &mut [PinState], val: PinState) {
        states[Self::IO_START..Self::IO_END].fill(val);
    }

    fn update(ram: &mut Vec<u8>, pins: &mut [PinState]) {
        let ce = !pins[Self::CE_INV];
        let oe = !pins[Self::OE_INV];
        let we = !pins[Self::WE_INV];

        let (_, output_pins) = pins.split_at_mut(Self::IO_START);
        let (io_pins, addr_pins) = output_pins.split_at_mut(Self::WORD_SIZE);

        let data = io_pins.iter().val().unwrap();
        let addr = addr_pins.iter().val().unwrap();

        if ce.is_lowish() || (oe.is_high() && we.is_high()) {
            Self::set_io(pins, PinState::HiZ);
        } else if oe.is_high() {
            let data = ram[addr];
            Self::set_output(io_pins, data as usize);
        } else if we.is_high() {
            ram[addr] = data as u8;
            Self::set_input(io_pins, data);
        }
    }

    fn set_output(pins: &mut [PinState], val: usize) {
        for (i, state) in pins.iter_mut().enumerate() {
            let bit = val & (1 << i);
            let signal = if bit == 0 { Signal::Low } else { Signal::High };
            *state = PinState::Output(signal)
        }
    }

    fn set_input(pins: &mut [PinState], val: usize) {
        for (i, state) in pins.iter_mut().enumerate() {
            let bit = val & (1 << i);
            let signal = if bit == 0 { Signal::Low } else { Signal::High };
            *state = PinState::Input(signal)
        }
    }
}

#[cfg(test)]
mod test_counter {
    use crate::*;
    use std::iter::zip;

    #[test]
    pub fn test_load() {
        let mut graph = Graph::new();

        let d = graph.new_pins(
            "d",
            &[
                PinState::Output(Signal::High),
                PinState::Output(Signal::Low),
                PinState::Output(Signal::Low),
                PinState::Output(Signal::High),
            ],
        );
        let mut load_inv = graph.new_output("load_inv", Signal::High);
        let mut clear = graph.new_output("clear", Signal::High);
        let up = graph.new_output("up", Signal::High);
        let down = graph.new_output("down", Signal::High);

        let counter = Ic74193::new(&mut graph, "counter");

        graph.connect(&up, counter.up());
        graph.connect(&down, counter.down());
        graph.connect(&load_inv, counter.load_inv());
        graph.connect(&clear, counter.clear());

        let connect_many = |seq: &[(&Pin, &Pin)]| {
            for (one, other) in seq {
                one.connect(other);
            }
        };

        connect_many(&[
            (&d[0], counter.in1()),
            (&d[1], counter.in2()),
            (&d[2], counter.in3()),
            (&d[3], counter.in4()),
        ]);

        graph.run();
        clear.set_output(Signal::Low);
        graph.run();

        assert_eq!(
            &[Signal::Low; 4],
            &[
                counter.out1().sig(),
                counter.out2().sig(),
                counter.out3().sig(),
                counter.out4().sig(),
            ]
        );

        load_inv.set_output(Signal::Low);
        graph.run();
        assert_eq!(
            &[Signal::High, Signal::Low, Signal::Low, Signal::High,],
            &[
                counter.out1().sig(),
                counter.out2().sig(),
                counter.out3().sig(),
                counter.out4().sig(),
            ]
        );
        load_inv.set_output(Signal::High);

        graph.run();
        assert_eq!(
            &[Signal::High, Signal::Low, Signal::Low, Signal::High,],
            &[
                counter.out1().sig(),
                counter.out2().sig(),
                counter.out3().sig(),
                counter.out4().sig(),
            ]
        );
    }

    #[test]
    pub fn test_carry() {
        let mut graph = Graph::new();

        let load_inv = graph.new_output("load_inv", Signal::High);
        let mut clear = graph.new_output("clear", Signal::High);
        let mut up = graph.new_output("up", Signal::High);
        let down = graph.new_output("down", Signal::High);

        let counter = Ic74193::new(&mut graph, "counter");

        graph.connect(&up, counter.up());
        graph.connect(&down, counter.down());

        let counter2 = Ic74193::new(&mut graph, "counter2");
        graph.connect(counter.carry(), counter2.up());
        graph.connect(counter.borrow(), counter2.down());
        graph.connect_all(&[&load_inv, counter.load_inv(), counter2.load_inv()]);
        graph.connect_all(&[&clear, counter.clear(), counter2.clear()]);

        graph.run();

        assert_eq!(
            &[Signal::Low; 8],
            &[
                counter.out1().sig(),
                counter.out2().sig(),
                counter.out3().sig(),
                counter.out4().sig(),
                counter2.out1().sig(),
                counter2.out2().sig(),
                counter2.out3().sig(),
                counter2.out4().sig(),
            ]
        );

        graph.set_output(&mut clear, Signal::Low);

        for _ in 0..16 {
            up.set_output(Signal::Low);
            graph.run();
            up.set_output(Signal::High);
            graph.run();
        }

        assert_eq!(
            &[
                Signal::Low,
                Signal::Low,
                Signal::Low,
                Signal::Low,
                Signal::High,
                Signal::Low,
                Signal::Low,
                Signal::Low,
            ],
            &[
                counter.out1().sig(),
                counter.out2().sig(),
                counter.out3().sig(),
                counter.out4().sig(),
                counter2.out1().sig(),
                counter2.out2().sig(),
                counter2.out3().sig(),
                counter2.out4().sig(),
            ],
        );
    }

    fn assert_sigs(pins: &[&Pin], sigs: &[Signal]) {
        assert_eq!(pins.len(), sigs.len());
        for (pin, sig) in zip(pins, sigs) {
            assert_eq!(pin.sig(), *sig, "{:?}", pin);
        }
    }

    #[test]
    pub fn test_counter_8bit() {
        let mut graph = Graph::new();

        let mut up = graph.new_output("up", Signal::High);
        let mut down = graph.new_output("down", Signal::High);
        let mut clear = graph.new_output("clear", Signal::High);
        let load_inv = graph.new_output("load_inv", Signal::High);

        let counter = Counter8Bit::new(&mut graph, "counter");

        graph.connect_pairs(&[
            (&up, counter.up()),
            (&down, counter.down()),
            (&clear, counter.clear()),
            (&load_inv, counter.load_inv()),
        ]);

        graph.run();
        clear.set_output(Signal::Low);
        graph.run();
        assert_sigs(
            &counter.output(),
            &[
                Signal::Low,
                Signal::Low,
                Signal::Low,
                Signal::Low,
                Signal::Low,
                Signal::Low,
                Signal::Low,
                Signal::Low,
            ],
        );

        up.set_output(Signal::Low);
        graph.run();
        up.set_output(Signal::High);
        graph.run();

        up.flash_output();
        graph.run();

        assert_sigs(
            &counter.output(),
            &[
                Signal::Low,
                Signal::High,
                Signal::Low,
                Signal::Low,
                Signal::Low,
                Signal::Low,
                Signal::Low,
                Signal::Low,
            ],
        );

        for _ in 1..10 {
            down.flash_output();
            graph.run();
        }

        assert_sigs(
            &counter.output(),
            &[
                Signal::High,
                Signal::Low,
                Signal::Low,
                Signal::High,
                Signal::High,
                Signal::High,
                Signal::High,
                Signal::High,
            ],
        );
    }

    #[test]
    pub fn test_counter_16bit() {
        let mut graph = Graph::new();

        let mut up = graph.new_output("up", Signal::High);
        let mut down = graph.new_output("down", Signal::High);
        let mut clear = graph.new_output("clear", Signal::High);
        let load_inv = graph.new_output("load_inv", Signal::High);

        let counter = Counter16Bit::new(&mut graph, "counter");

        graph.connect_pairs(&[
            (&up, counter.up()),
            (&down, counter.down()),
            (&clear, counter.clear()),
            (&load_inv, counter.load_inv()),
        ]);

        graph.run();
        clear.set_output(Signal::Low);
        graph.run();
        assert_sigs(&counter.output(), &[Signal::Low; 16]);

        up.set_output(Signal::Low);
        graph.run();
        up.set_output(Signal::High);
        graph.run();

        up.flash_output();
        graph.run();

        assert_sigs(
            &counter.output(),
            &[
                Signal::Low,
                Signal::High,
                Signal::Low,
                Signal::Low,
                Signal::Low,
                Signal::Low,
                Signal::Low,
                Signal::Low,
                Signal::Low,
                Signal::Low,
                Signal::Low,
                Signal::Low,
                Signal::Low,
                Signal::Low,
                Signal::Low,
                Signal::Low,
            ],
        );

        for _ in 1..10 {
            down.flash_output();
            graph.run();
        }

        assert_sigs(
            &counter.output(),
            &[
                Signal::High,
                Signal::Low,
                Signal::Low,
                Signal::High,
                Signal::High,
                Signal::High,
                Signal::High,
                Signal::High,
                Signal::High,
                Signal::High,
                Signal::High,
                Signal::High,
                Signal::High,
                Signal::High,
                Signal::High,
                Signal::High,
            ],
        );
    }
}

#[cfg(test)]
mod test_ram {
    use crate::*;
    use std::iter::zip;

    fn assert_states(pins: &[Pin], states: &[PinState]) {
        assert_eq!(pins.len(), states.len());
        for (pin, state) in zip(pins, states) {
            assert_eq!(pin.state(), *state, "{:?}", pin);
        }
    }

    fn assert_inputs(pins: &[Pin]) {
        for pin in pins {
            assert!(matches!(pin.state(), PinState::Input(_)));
        }
    }

    fn assert_outputs(pins: &[Pin]) {
        for pin in pins {
            assert!(matches!(pin.state(), PinState::Output(_)));
        }
    }

    #[test]
    pub fn test_read_write() {
        let mut graph = Graph::new();

        let ram = IcCY7C199::new(&mut graph, "d_ram");

        graph.run();
        assert_states(ram.io(), &[PinState::HiZ; 8]);

        let ce_inv = graph.new_output("ce_inv", Signal::Low);
        let mut oe_inv = graph.new_output("oe_inv", Signal::Low);
        let mut we_inv = graph.new_output("we_inv", Signal::High);
        graph.connect_pairs(&[
            (&ce_inv, ram.ce_inv()),
            (&oe_inv, ram.oe_inv()),
            (&we_inv, ram.we_inv()),
        ]);

        graph.run();

        assert_outputs(ram.io());

        let mut d = graph.new_pins("d", &[PinState::Output(Signal::Low); 8]);
        for (one, other) in zip(&d, ram.io()) {
            graph.connect(one, other);
        }

        oe_inv.set_output(Signal::High);
        graph.run();

        we_inv.set_output(Signal::Low);
        graph.run();

        assert_inputs(ram.io());

        we_inv.set_output(Signal::High);
        oe_inv.set_output(Signal::Low);
        graph.run();

        let mut expected_outputs = [PinState::Output(Signal::Low); 8];
        assert_states(ram.io(), &expected_outputs);

        d[2].set_output(Signal::High);
        expected_outputs[2] = PinState::Output(Signal::High);
        oe_inv.set_output(Signal::High);
        we_inv.set_output(Signal::Low);
        graph.run();
        we_inv.set_output(Signal::High);
        oe_inv.set_output(Signal::Low);
        graph.run();

        assert_states(ram.io(), &expected_outputs);

        let mut a2 = graph.new_output("a2", Signal::High);
        graph.connect(&a2, &ram.addr()[2]);
        graph.run();

        assert_states(ram.io(), &[PinState::Output(Signal::High); 8]);

        a2.set_output(Signal::Low);
        graph.run();

        assert_states(ram.io(), &expected_outputs);
    }
}
