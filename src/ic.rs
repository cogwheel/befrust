use crate::*;

/// T flip-flop used for 74193 counter memory
#[derive(Debug)]
pub struct TFlipFlop {
    toggle: Pin,
    set: Pin,
    reset: Pin,
    output: Pin,
    out_inv: Pin,
}

impl TFlipFlop {
    /// Toggle pin
    ///
    /// Unless reset or set are high, output will change states when the toggle changes from low to
    /// high
    pub fn toggle(&self) -> &Pin {
        &self.toggle
    }

    /// Set pin
    ///
    /// Unless reset is High, output will change to High when Set is High
    pub fn set(&self) -> &Pin {
        &self.set
    }

    /// Reset pin
    ///
    /// Output will change to Low when Reset is High
    pub fn reset(&self) -> &Pin {
        &self.reset
    }

    /// Output pin
    pub fn output(&self) -> &Pin {
        &self.output
    }

    /// Inverted output pin
    pub fn out_inv(&self) -> &Pin {
        &self.out_inv
    }

    // external
    /// Toggle pin index
    const TOGGLE: usize = 0;

    /// Set pin index
    const SET: usize = 1;

    /// Reset pin index
    const RESET: usize = 2;

    /// Output pin index
    const OUTPUT: usize = 3;

    /// Inverted output pin index
    const OUT_INV: usize = 4;

    // internal for edge detection
    /// Previous toggle state index
    const TOGGLE_PREV: usize = 5;

    /// Create a T-flip flop
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

/// The low order bit memory and counting logic for the 74193
#[derive(Debug)]
pub struct HalfAdder {
    input: Pin,
    clear: Pin,
    load: Pin,
    flip_flop: TFlipFlop,
}

impl HalfAdder {
    /// Input pin
    ///
    /// Data to be loaded if load occurs
    pub fn input(&self) -> &Pin {
        &self.input
    }

    /// Clear pin
    ///
    /// Output will reset to 0 when this is high
    pub fn clear(&self) -> &Pin {
        &self.clear
    }

    /// Load pin
    ///
    /// Output will be set to value of input pin when this goes high
    pub fn load(&self) -> &Pin {
        &self.load
    }

    /// Toggle pin
    pub fn toggle(&self) -> &Pin {
        self.flip_flop.toggle()
    }

    /// Output pin
    pub fn output(&self) -> &Pin {
        self.flip_flop.output()
    }

    /// Inverted output pin
    pub fn out_inv(&self) -> &Pin {
        self.flip_flop.out_inv()
    }

    /// Create a new half adder
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

/// The higher-order bit memory and counting logic for the 74193
#[derive(Debug)]
pub struct FullAdder {
    up: Pin,
    down: Pin,
    up_cond: Pin,
    down_cond: Pin,
    half_adder: HalfAdder,
}

impl FullAdder {
    /// Input up count signal
    pub fn up(&self) -> &Pin {
        &self.up
    }

    /// Input down count signal
    pub fn down(&self) -> &Pin {
        &self.down
    }

    /// Condition for when the up count should be observed
    pub fn up_cond(&self) -> &Pin {
        &self.up_cond
    }

    /// Condition for when the down count should be observed
    pub fn down_cond(&self) -> &Pin {
        &self.down_cond
    }

    /// Input pin used for loading data
    pub fn input(&self) -> &Pin {
        &self.half_adder.input()
    }

    /// Clear pin resets the bit to 0
    pub fn clear(&self) -> &Pin {
        self.half_adder.clear()
    }

    /// Load pin causes bit to be set to state of `input`
    pub fn load(&self) -> &Pin {
        self.half_adder.load()
    }

    /// The state of the bit
    pub fn output(&self) -> &Pin {
        self.half_adder.output()
    }

    /// The inverted state of the bit
    pub fn out_inv(&self) -> &Pin {
        self.half_adder.out_inv()
    }

    /// Create a new full adder
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

/// Implementation of the 74193 chip
///
/// Based on schematic from <https://www.ti.com/lit/ds/symlink/sn54ls193-sp.pdf?ts=1649956332500>
pub struct Ic74193 {
    // inputs
    up: Pin,
    down: Pin,
    load_inv: Pin,
    clear: Pin,

    // outputs
    borrow: Pin,
    carry: Pin,

    adder1: HalfAdder,
    adder2: FullAdder,
    adder3: FullAdder,
    adder4: FullAdder,
}

impl Ic74193 {
    /// Input up count signal
    ///
    /// Normally high; when transitioning to High, causes counter to increase by one
    pub fn up(&self) -> &Pin {
        &self.up
    }

    /// Input down count signal
    ///
    /// Normally held high. When transitioning to high (after going low), causes counter to count down
    pub fn down(&self) -> &Pin {
        &self.down
    }

    /// Input inverted load signal
    ///
    /// When brought low, the data in the counter is set to the value on the `input`
    pub fn load_inv(&self) -> &Pin {
        &self.load_inv
    }

    /// Clear signal
    ///
    /// When high, the data in the counter is reset to 0
    pub fn clear(&self) -> &Pin {
        &self.clear
    }

    /// The input pins (d1-d4)
    pub fn input(&self) -> [&Pin; 4] {
        [self.in1(), self.in2(), self.in3(), self.in4()]
    }

    /// Input pin 1 (d1)
    pub fn in1(&self) -> &Pin {
        self.adder1.input()
    }

    /// Input pin 2 (d2)
    pub fn in2(&self) -> &Pin {
        self.adder2.input()
    }

    /// Input pin 3 (d3)
    pub fn in3(&self) -> &Pin {
        self.adder3.input()
    }

    /// Input pin 4 (d4)
    pub fn in4(&self) -> &Pin {
        self.adder4.input()
    }

    /// Carry output
    ///
    /// Normally high. If counter is 15, transitions to low, then High, following the up signal
    pub fn carry(&self) -> &Pin {
        &self.carry
    }

    /// Borrow output
    ///
    /// Normally high. If counter is 0, transitions to low, then High, following the down signal
    pub fn borrow(&self) -> &Pin {
        &self.borrow
    }

    /// The output pins (q1-q4)
    pub fn output(&self) -> [&Pin; 4] {
        [self.out1(), self.out2(), self.out3(), self.out4()]
    }

    /// Output pin 1 (q1)
    pub fn out1(&self) -> &Pin {
        self.adder1.output()
    }

    /// Output pin 2 (q2)
    pub fn out2(&self) -> &Pin {
        self.adder2.output()
    }

    /// Output pin 3 (q3)
    pub fn out3(&self) -> &Pin {
        self.adder3.output()
    }

    /// Output pin 4 (q4)
    pub fn out4(&self) -> &Pin {
        self.adder4.output()
    }

    /// Create a new Ic74193
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

/// 8-bit counter made of a pair of 74193s
pub struct Counter8Bit {
    counter1: Ic74193,
    counter2: Ic74193,
}

impl Counter8Bit {
    /// Input pins for loading new data
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

    /// Count up signal
    pub fn up(&self) -> &Pin {
        self.counter1.up()
    }

    /// Count down signal
    pub fn down(&self) -> &Pin {
        self.counter1.down()
    }

    /// Inverted load signal
    pub fn load_inv(&self) -> &Pin {
        self.counter1.load_inv()
    }

    /// Clear signal
    pub fn clear(&self) -> &Pin {
        self.counter1.clear()
    }

    /// Output data
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

    /// Output carry
    pub fn carry(&self) -> &Pin {
        self.counter2.carry()
    }

    /// Output borrow
    pub fn borrow(&self) -> &Pin {
        self.counter2.borrow()
    }

    /// Create a new Counter8Bit
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
    /// Input bits for loading data
    pub fn input(&self) -> [&Pin; 16] {
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

    /// Count up signal
    pub fn up(&self) -> &Pin {
        self.counter1.up()
    }

    /// Count down signal
    pub fn down(&self) -> &Pin {
        self.counter1.down()
    }

    /// Inverted load signal
    pub fn load_inv(&self) -> &Pin {
        self.counter1.load_inv()
    }

    /// Clear signal
    pub fn clear(&self) -> &Pin {
        self.counter1.clear()
    }

    /// Output pins
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

    /// Output carry
    pub fn carry(&self) -> &Pin {
        self.counter2.carry()
    }

    /// Output borrow
    pub fn borrow(&self) -> &Pin {
        self.counter2.borrow()
    }

    /// Create a new Counter16Bit
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

/// 32k x 8bit RAM modeled after the CY7C199
pub struct IcCY7C199(Vec<Pin>);

impl IcCY7C199 {
    /// Inverse chip enable pin index
    const CE_INV: usize = 0;

    /// Inverse output enable pin index
    const OE_INV: usize = 1;

    /// Inverse write enable pin index
    const WE_INV: usize = 2;

    /// IO pin starting index
    const IO_START: usize = 3;

    /// Size of word (number of IO pins)
    const WORD_SIZE: usize = 8;

    /// IO pin ending index
    const IO_END: usize = Self::IO_START + Self::WORD_SIZE;

    /// Address pin starting index
    const ADDR_START: usize = Self::IO_END;

    /// Address width
    const ADDR_SIZE: usize = 15;

    /// Address pin ending index
    const ADDR_END: usize = Self::ADDR_START + Self::ADDR_SIZE;

    /// Total number of pins in part
    const NUM_PINS: usize = Self::ADDR_END;

    /// Total number of words in RAM
    const NUM_WORDS: usize = 1 << Self::ADDR_SIZE;

    /// Inverted chip enable
    pub fn ce_inv(&self) -> &Pin {
        &self.0[Self::CE_INV]
    }

    /// Inverted output enable
    pub fn oe_inv(&self) -> &Pin {
        &self.0[Self::OE_INV]
    }

    /// Inverted write enable
    pub fn we_inv(&self) -> &Pin {
        &self.0[Self::WE_INV]
    }

    /// I/O pins
    ///
    /// Pins are set to HiZ if:
    ///     `ce_inv` is High, or
    ///     `oe_inv` and `we_inv` are both Low
    ///
    /// Otherwise, pins are set to output the contents of ram at the current address if `oe_inv` is
    ///     Low
    ///
    /// Otherwise, pins are set to input and their value is written into ram if `we_inv` is Low
    pub fn io(&self) -> &[Pin] {
        &self.0[Self::IO_START..Self::IO_END]
    }

    /// Address pins
    pub fn addr(&self) -> &[Pin] {
        &self.0[Self::ADDR_START..Self::ADDR_END]
    }

    /// Create a new RAM part
    pub fn new(graph: &mut Graph, name: &str) -> Self {
        let mut states = [PinState::INPUT; Self::NUM_PINS];
        Self::set_io(&mut states, PinState::HiZ);

        // TODO: Need a way to examine this vector
        // TODO: Randomize the contents
        let mut ram = vec![0xff; Self::NUM_WORDS];

        let pins = graph.new_part(name, &states, move |pins| {
            Self::update(&mut ram, pins);
        });

        Self(pins)
    }

    /// Sets the IO pins to a given state
    fn set_io(states: &mut [PinState], val: PinState) {
        states[Self::IO_START..Self::IO_END].fill(val);
    }

    /// Part updater
    fn update(ram: &mut Vec<u8>, pins: &mut [PinState]) {
        let ce = !pins[Self::CE_INV];
        let oe = !pins[Self::OE_INV];
        let we = !pins[Self::WE_INV];

        let (_, output_pins) = pins.split_at_mut(Self::IO_START);
        let (io_pins, addr_pins) = output_pins.split_at_mut(Self::WORD_SIZE);

        let data = io_pins.iter().val().unwrap();
        let addr = addr_pins.iter().val().unwrap();

        if ce.is_lowish() {
            Self::set_io(pins, PinState::HiZ);
        } else if oe.is_high() {
            let data = ram[addr];
            Self::set_output(io_pins, data as usize);
        } else {
            Self::set_input(io_pins, data);

            if we.is_high() {
                //println!("writing {} to {}", data, addr);
                ram[addr] = data as u8;
            }
        }
    }

    /// Sets the pins to output the given value
    fn set_output(pins: &mut [PinState], val: usize) {
        let bus_val = BusValue::new_val(val);
        for (i, state) in pins.iter_mut().enumerate() {
            *state = PinState::Output(bus_val.sig(i));
        }
    }

    /// Changes the pins to inputs and sets them to the given value
    fn set_input(pins: &mut [PinState], val: usize) {
        let bus_val = BusValue::new_val(val);
        for (i, state) in pins.iter_mut().enumerate() {
            *state = PinState::Input(bus_val.sig(i));
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

        let mut d = graph.new_pins(
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

        // make sure the module isn't just looping back the input. Yes this actually happened -_-
        for pin in d.iter_mut() {
            pin.set_output(Signal::Low);
        }
        graph.run();

        assert_eq!(
            &[
                counter.output()[0].sig(),
                counter.output()[1].sig(),
                counter.output()[2].sig(),
                counter.output()[3].sig(),
            ],
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
