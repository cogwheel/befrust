use befrust::*;
use derive_getters::Getters;

#[derive(Debug, Getters)]
pub struct TFlipFlop{
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
        TFlipFlop{
            t: pins[Self::T].clone(),
            s: pins[Self::S].clone(),
            r: pins[Self::R].clone(),
            q: pins[Self::Q].clone(),
            q_inv: pins[Self::Q_INV].clone(),
        }
    }
}

#[derive(Debug, Getters)]
pub struct AdderImpl {
    d: Pin,
    clear: Pin,
    load: Pin,

    #[getter(skip)]
    flip_flop: TFlipFlop,
}

impl AdderImpl {
    pub fn t(&self) -> &Pin { self.flip_flop.t() }
    pub fn q(&self) -> &Pin { self.flip_flop.q() }
    pub fn q_inv(&self) -> &Pin { self.flip_flop.q_inv() }

    pub fn new(graph: &mut Graph, name: &str) -> AdderImpl {
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

        AdderImpl {d, clear, load, flip_flop}
    }
}

#[derive(Debug, Getters)]
struct FullAdder {
    up: Pin,
    down: Pin,
    up_cond: Pin,
    down_cond: Pin,

    #[getter(skip)]
    adder_impl: AdderImpl,
}

impl FullAdder {
    pub fn d(&self) -> &Pin { &self.adder_impl.d() }
    pub fn clear(&self) -> &Pin { self.adder_impl.clear() }
    pub fn load(&self) -> &Pin { self.adder_impl.load() }

    const UP: usize = 0;
    const DOWN: usize = 1;
    const UP_COND: usize = 2;
    const DOWN_COND: usize = 3;

    pub fn new(graph: &mut Graph, name: &str) -> FullAdder {
        let adder = FullAdder{
            up: graph.new_input(&format!("{}.up", name)),
            down: graph.new_input(&format!("{}.down", name)),
            up_cond: graph.new_input(&format!("{}.up_cond", name)),
            down_cond: graph.new_input(&format!("{}.down_cond", name)),
            adder_impl: AdderImpl::new(graph, &format!("{}.impl", name)),
        };

        let toggle = !(adder.up() & adder.up_cond() | adder.down() & adder.down_cond());

        graph.connect(&toggle, adder.adder_impl.t());

        adder
    }
}

// TODO: make brainfuck computer
fn main() {
    // also nyi
    let mut graph = Graph::new();

    let d1 = graph.new_output("d1", Signal::High);
    let load = graph.new_output("load", Signal::Low);
    let clear = graph.new_output("clear", Signal::Low);
    let up = graph.new_output("up", Signal::Low);
    let down = graph.new_output("down", Signal::Low);

    let adder1 = AdderImpl::new(&mut graph, "adder1");
    let toggle1 = !(&up | &down);

    graph.connect(&d1, adder1.d());
    graph.connect(&clear, &!adder1.clear());
    graph.connect(&load, adder1.load());
    graph.connect(&toggle1, adder1.t());

    let mut end = graph.new_output("start", Signal::Low);
    for _ in 0..20_000 {
        end = !end;
    }

    println!("{:?}", adder1.q());
    graph.run();
    println!("{:?}", adder1.q());

    graph.set_output(&up, Signal::High);
    graph.run();
    println!("{:?}", adder1.q());

    graph.set_output(&up, Signal::Low);
    graph.run();

    println!("{:?}", adder1.q());
}
