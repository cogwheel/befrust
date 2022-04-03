use crate::*;
use derive_getters::Getters;

#[derive(Debug, Getters)]
pub struct TFlipFlop {
    t: Pin,
    s: Pin,
    r: Pin,
    q: Pin,
    q_inv: Pin,
}

impl TFlipFlop {
    // external
    const T: usize = 0;
    const S: usize = 1;
    const R: usize = 2;
    const Q: usize = 3;
    const Q_INV: usize = 4;

    // internal
    const T_PREV: usize = 5;

    pub fn new(graph: &mut Graph, name: &str) -> TFlipFlop {
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
            |before, after| {
                let new_q = if before[Self::R].is_high() {
                    Signal::Low
                } else if before[Self::S].is_high() {
                    Signal::High
                } else if before[Self::T].is_high() && before[Self::T_PREV].is_lowish() {
                    before[Self::Q_INV].sig()
                } else {
                    before[Self::Q].sig()
                };
                after[Self::Q] = PinState::Output(new_q);
                after[Self::Q_INV] = PinState::Output(!new_q);
                after[Self::T_PREV] = before[Self::T];
            },
        );
        TFlipFlop {
            t: pins[Self::T].clone(),
            s: pins[Self::S].clone(),
            r: pins[Self::R].clone(),
            q: pins[Self::Q].clone(),
            q_inv: pins[Self::Q_INV].clone(),
        }
    }
}

#[derive(Debug, Getters)]
pub struct HalfAdder {
    d: Pin,
    clear: Pin,
    load: Pin,

    #[getter(skip)]
    flip_flop: TFlipFlop,
}

impl HalfAdder {
    pub fn t(&self) -> &Pin {
        self.flip_flop.t()
    }
    pub fn q(&self) -> &Pin {
        self.flip_flop.q()
    }
    pub fn q_inv(&self) -> &Pin {
        self.flip_flop.q_inv()
    }

    pub fn new(graph: &mut Graph, name: &str) -> HalfAdder {
        let d = graph.new_input(&format!("{}.d", name));
        let clear = graph.new_input(&format!("{}.clear", name));
        let load = graph.new_input(&format!("{}.load", name));
        let flip_flop = TFlipFlop::new(graph, &format!("{}.flip_flop", name));

        let input = !(&d & &load & clear.clone());
        let set = !&input;

        let load_ff = !(&input & &load);
        let reset_ff = !&clear | !load_ff;

        graph.connect(&set, flip_flop.s());
        graph.connect(&reset_ff, flip_flop.r());

        HalfAdder {
            d,
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

    // TODO: figure out how to do #[getter(forward)]
    // TODO: figure out how to contact the author of derive-getters
    #[getter(skip)]
    half_adder: HalfAdder,
}

impl FullAdder {
    pub fn d(&self) -> &Pin {
        &self.half_adder.d()
    }
    pub fn clear(&self) -> &Pin {
        self.half_adder.clear()
    }
    pub fn load(&self) -> &Pin {
        self.half_adder.load()
    }
    pub fn q(&self) -> &Pin {
        self.half_adder.q()
    }
    pub fn q_inv(&self) -> &Pin {
        self.half_adder.q_inv()
    }

    pub fn new(graph: &mut Graph, name: &str) -> FullAdder {
        let adder = FullAdder {
            up: graph.new_input(&format!("{}.up", name)),
            down: graph.new_input(&format!("{}.down", name)),
            up_cond: graph.new_input(&format!("{}.up_cond", name)),
            down_cond: graph.new_input(&format!("{}.down_cond", name)),
            half_adder: HalfAdder::new(graph, &format!("{}.half", name)),
        };

        let toggle = !(adder.up() & adder.up_cond() | adder.down() & adder.down_cond());

        graph.connect(&toggle, adder.half_adder.t());

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
    pub fn d(&self) -> [&Pin; 4] {
        [self.d1(), self.d2(), self.d3(), self.d4()]
    }
    pub fn d1(&self) -> &Pin {
        self.adder1.d()
    }
    pub fn d2(&self) -> &Pin {
        self.adder2.d()
    }
    pub fn d3(&self) -> &Pin {
        self.adder3.d()
    }
    pub fn d4(&self) -> &Pin {
        self.adder4.d()
    }

    pub fn q(&self) -> [&Pin; 4] {
        [self.d1(), self.d2(), self.d3(), self.d4()]
    }
    pub fn q1(&self) -> &Pin {
        self.adder1.q()
    }
    pub fn q2(&self) -> &Pin {
        self.adder2.q()
    }
    pub fn q3(&self) -> &Pin {
        self.adder3.q()
    }
    pub fn q4(&self) -> &Pin {
        self.adder4.q()
    }

    pub fn new(graph: &mut Graph, name: &str) -> Ic74193 {
        let up_inv = not_gate(graph, &format!("{}.up_inv", name));
        let down_inv = not_gate(graph, &format!("{}.down_inv", name));
        let load = not_gate(graph, &format!("{}.load", name));
        let clear_inv = not_gate(graph, &format!("{}.clear_inv", name));
        let adder1 = HalfAdder::new(graph, &format!("{}.adder1", name));
        let adder2 = FullAdder::new(graph, &format!("{}.adder2", name));
        let adder3 = FullAdder::new(graph, &format!("{}.adder3", name));
        let adder4 = FullAdder::new(graph, &format!("{}.adder4", name));

        let carry = nand_nary(graph, "carry", 5);
        carry.connect_inputs(&[up_inv.q(), adder1.q(), adder2.q(), adder3.q(), adder4.q()]);

        let borrow = nand_nary(graph, "borrow", 5);
        borrow.connect_inputs(&[
            down_inv.q(),
            adder1.q_inv(),
            adder2.q_inv(),
            adder3.q_inv(),
            adder4.q_inv(),
        ]);

        let toggle1 = !(up_inv.q() | down_inv.q());

        toggle1.connect(adder1.t());

        let up_cond2 = adder1.q();
        let down_cond2 = adder1.q_inv();
        graph.connect(&up_cond2, adder2.up_cond());
        graph.connect(&down_cond2, adder2.down_cond());

        let up_cond3 = up_cond2 & adder2.q();
        let down_cond3 = down_cond2 & adder2.q_inv();
        graph.connect(&up_cond3, adder3.up_cond());
        graph.connect(&down_cond3, adder3.down_cond());

        let up_cond4 = &up_cond3 & adder3.q();
        let down_cond4 = &down_cond3 & adder3.q_inv();
        graph.connect(&up_cond4, adder4.up_cond());
        graph.connect(&down_cond4, adder4.down_cond());

        up_inv
            .q()
            .connect_all(&[adder2.up(), adder3.up(), adder4.up()]);

        down_inv
            .q()
            .connect_all(&[adder2.down(), adder3.down(), adder4.down()]);

        load.q()
            .connect_all(&[adder1.load(), adder2.load(), adder3.load(), adder4.load()]);

        clear_inv.q().connect_all(&[
            adder1.clear(),
            adder2.clear(),
            adder3.clear(),
            adder4.clear(),
        ]);

        Ic74193 {
            up: up_inv.a().clone(),
            down: down_inv.a().clone(),
            load_inv: load.a().clone(),
            clear: clear_inv.a().clone(),
            carry: carry.q().clone(),
            borrow: borrow.q().clone(),
            adder1,
            adder2,
            adder3,
            adder4,
        }
    }
}

#[cfg(test)]
mod test_counter {
    use crate::*;

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
            (&d[0], counter.d1()),
            (&d[1], counter.d2()),
            (&d[2], counter.d3()),
            (&d[3], counter.d4()),
        ]);

        graph.run();
        clear.set_output(Signal::Low);
        graph.run();

        assert_eq!(
            &[Signal::Low; 4],
            &[
                counter.q1().sig(),
                counter.q2().sig(),
                counter.q3().sig(),
                counter.q4().sig(),
            ]
        );

        load_inv.set_output(Signal::Low);
        graph.run();
        assert_eq!(
            &[Signal::High, Signal::Low, Signal::Low, Signal::High,],
            &[
                counter.q1().sig(),
                counter.q2().sig(),
                counter.q3().sig(),
                counter.q4().sig(),
            ]
        );
        load_inv.set_output(Signal::High);

        graph.run();
        assert_eq!(
            &[Signal::High, Signal::Low, Signal::Low, Signal::High,],
            &[
                counter.q1().sig(),
                counter.q2().sig(),
                counter.q3().sig(),
                counter.q4().sig(),
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
                counter.q1().sig(),
                counter.q2().sig(),
                counter.q3().sig(),
                counter.q4().sig(),
                counter2.q1().sig(),
                counter2.q2().sig(),
                counter2.q3().sig(),
                counter2.q4().sig(),
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
                counter.q1().sig(),
                counter.q2().sig(),
                counter.q3().sig(),
                counter.q4().sig(),
                counter2.q1().sig(),
                counter2.q2().sig(),
                counter2.q3().sig(),
                counter2.q4().sig(),
            ],
        );
    }
}
