use befrust::*;
use std::io::Write;

fn main() {
    #![allow(unused_assignments, unused_mut)]

    let mut graph = Graph::new();

    // Create signals to send as inputs to the data block
    //
    // these will be outputs of the control block eventually

    // Increment direction
    let mut down = graph.new_output("down", Signal::Low);
    let up = !&down;

    // clock phases
    //
    // TODO:Instruction - Latches the next instruction and increments program counter
    //
    // Count - increments any enabled counters (pointers, registers, etc.)
    // Store - commits any changes (e.g. writing ram after data increments)
    // Clear - only active during reset, used to step through RAM addresses
    let mut count = graph.new_output("count", Signal::Low);
    let mut store = graph.new_output("store", Signal::Low);
    let mut clear = graph.new_output("clear", Signal::Low);

    // Enable lines
    let mut p_ce = graph.new_output("p_ce", Signal::Low);
    let mut d_ce = graph.new_output("d_ce", Signal::Low);

    let mut reset = graph.new_output("reset", Signal::High);

    let d_block = DataBlock::new(&mut graph, "data");

    // Connect our manual signals to the data block
    graph.connect_pairs(&[
        (&up, d_block.up()),
        (&down, d_block.down()),
        (&count, d_block.count()),
        (&store, d_block.store()),
        (&p_ce, d_block.p_ce()),
        (&d_ce, d_block.d_ce()),
        (&reset, d_block.reset()),
        (&clear, d_block.clear()),
    ]);

    // The zero flag constantly reads from the bus for use in control signals
    let zero = nor_nary(&mut graph, "zero", d_block.data().len());

    let print_debug =
        |m: &str| println!("{}: {:?}, z:{:?}", m, d_block, zero.output().state().val());

    print_debug("connected graph");

    ////// Use the graph

    dbg!(graph.run());
    print_debug("first_run");

    // TODO: need to optimize so we can actually clear all of RAM
    //
    // For now ust do a couple clears to verify that the circuitry is working
    const NUM_CLEARS: i32 = 5; // should be at least the size of ram (1 << 15)

    // This is a cheat for now. IRL the clear clock would always be running but ignored while reset
    // is off
    println!("Clearing RAM");
    //println!("This can take a while on debug builds");
    for i in 0..NUM_CLEARS {
        graph.pulse_output(&mut clear);
        if i % 1000 == 0 {
            print!("{}% ", ((i as f32 / (1 << 15) as f32) * 100.0) as i32);
            std::io::stdout().flush().unwrap();
        }
    }
    println!("100%");

    print_debug("end of loop");

    clear.set_output(Signal::High);
    dbg!(graph.run());
    clear.set_output(Signal::Low);
    dbg!(graph.run());

    print_debug("end of clear");

    // At this point, the address pointer is at some arbitrary location depending on how long the
    // reset line is held (i.e. how many ticks of the clear count happened while reset was active).
    // IRL the clear clock will be in the 2 MHz range which can clear all of RAM in a few dozen ms.

    // TODO: maybe reset ram address to 0 at the end of reset, but isn't necessary since counters
    // wrap. It would be useful for debugging to have programs always start at 0 though...

    // Test data reg
    reset.set_output(Signal::Low);
    graph.run();

    print_debug("end reset");

    d_ce.set_output(Signal::High);
    graph.run();
    print_debug("d_ce high");

    count.set_output(Signal::High);
    graph.run();
    print_debug("count high");

    count.set_output(Signal::Low);
    graph.run();
    print_debug("count low");

    graph.pulse_output(&mut count);
    print_debug("count pulse");

    store.set_output(Signal::High);
    dbg!(graph.run());
    print_debug("store high");

    store.set_output(Signal::Low);
    graph.run();
    print_debug("store low");

    // test data ptr
    d_ce.set_output(Signal::Low);
    graph.run();
    print_debug("d_ce low");

    p_ce.set_output(Signal::High);
    graph.run();
    print_debug("p_ce high");

    graph.pulse_output(&mut count);

    print_debug("ptr pulse 1");
    count.set_output(Signal::High);
    graph.run();
    count.set_output(Signal::Low);
    graph.run();
    print_debug("ptr pulse 2");

    down.set_output(Signal::High);

    count.set_output(Signal::High);
    graph.run();
    count.set_output(Signal::Low);
    graph.run();

    graph.pulse_output(&mut count);
    graph.pulse_output(&mut count);
    graph.pulse_output(&mut count);

    count.set_output(Signal::High);
    graph.run();
    count.set_output(Signal::Low);
    graph.run();

    print_debug("ptr down 5");

    down.set_output(Signal::Low);
    graph.pulse_output(&mut count);
    graph.pulse_output(&mut count);
    graph.pulse_output(&mut count);
    graph.pulse_output(&mut count);
    graph.pulse_output(&mut store);

    print_debug("ptr up 4");

    p_ce.set_output(Signal::Low);
    graph.run();
    print_debug("pe low");

    d_ce.set_output(Signal::High);

    graph.run();
    print_debug("p_ce/d_ce swap");

    graph.pulse_output(&mut count);

    print_debug("data count up");

    graph.pulse_output(&mut store);

    print_debug("data store");
}
