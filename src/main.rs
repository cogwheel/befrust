use befrust::*;


// TODO: make brainfuck computer
fn main() {
    // also nyi
    let mut graph = Graph::new();
    let a = graph.new_const(Signal::Low);
    let b = graph.new_const(Signal::High);

    /*
    let a_and_b = a & b; // returns a pin or a part?
    let not_a = !a;
    let a_or_b = a | b;
    let a_xor_b = a ^ b;

    let a_shift = a >> 1;

    let latch = RsLatch::new(&mut graph);
    let ff = graph.new_part(
        "t_flip_flop",
        &[
            ("t", PinState::INPUT),
            ("s", PinState::INPUT),
            ("r", PinState::INPUT),
            ("q", PinState::Output(Signal::Low)),
            ("not_q", PinState::Output(Signal::High)),
        ],
        move |&[input], &mut [output]|{
            let &[t, s, r, q, not_q] = input;
            m
        }
    );
    // ff = GenericPart{name: "t_flip_flop_0123", pins: Vec[Pin{id:1}, Pin{id:2}, ...]}
    // either:
    graph.connect(a_and_b.q, flip_flop.r);
    // or:
    graph.connect(a_and_b, flip_flop.r); // either "single-out" concept or above things return a pin, not a component

    // impl Source for BinaryGate
    // impl Source for Pin

    // operators take an OutputPin trait.

     */
}
