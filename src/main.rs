use befrust::*;
use std::fmt::{Debug, Formatter};
use std::io::Write;
use std::iter::zip; // for flush

pub struct DataBlock {
    d_ce: Pin,
    p_ce: Pin,
    up: Pin,
    down: Pin,
    count: Pin,
    store: Pin,
    reset: Pin,
    clear: Pin,
    bus: BusBuffer,
    ptr: Counter16Bit,
}

impl Debug for DataBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataBlock")
            .field("data", &self.data().iter().val())
            .field("addr", &self.addr().iter().val())
            .finish()
    }
}

impl DataBlock {
    pub fn d_ce(&self) -> &Pin {
        &self.d_ce
    }
    pub fn p_ce(&self) -> &Pin {
        &self.p_ce
    }
    pub fn up(&self) -> &Pin {
        &self.up
    }
    pub fn down(&self) -> &Pin {
        &self.down
    }
    pub fn count(&self) -> &Pin {
        &self.count
    }
    pub fn store(&self) -> &Pin {
        &self.store
    }
    pub fn reset(&self) -> &Pin {
        &self.reset
    }
    pub fn clear(&self) -> &Pin {
        &self.clear
    }

    pub fn data(&self) -> &[Pin] {
        &self.bus.output()
    }

    pub fn addr(&self) -> [&Pin; 16] {
        self.ptr.output()
    }

    pub fn new(graph: &mut Graph, name: &str) -> Self {
        let make_name = |n: &str| format!("{}.{}", name, n);

        // `ptr` stores the address for operations `<` and `>`
        let ptr = Counter16Bit::new(graph, &make_name("ptr"));

        // `reg` stores the current working counter for operations `+` and `-`
        // This is transferred to/from `ram` as needed when `ptr` changes
        let reg = Counter8Bit::new(graph, &make_name("reg"));

        let ram = IcCY7C199::new(graph, &make_name("ram"));
        // Connect pointer outputs to the ram address lines
        for (ptr_pin, ram_pin) in zip(ptr.output(), ram.addr()) {
            ptr_pin.connect(ram_pin);
        }

        // TODO: un-hardcode the 8s? then again... u8 is used everywhere

        // Main data bus, also connected to other components (e.g. I/O)
        let bus = BusBuffer::new(graph, &make_name("bus"), 8);

        // Allows reg to be connected bidirectionally to or disconnected from the bus
        let reg_interface = BusTristate::new(graph, &make_name("reg_interface"), 8);

        // Connect everything to the bus input. The bus output is the external interface
        for i in 0..8 {
            graph.connect_all(&[
                &bus.input()[i],
                &ram.io()[i],
                reg.input()[i],
                &reg_interface.output()[i],
            ]);
            graph.connect(&reg.output()[i], &reg_interface.input()[i]);
        }

        let up = graph.new_input(&make_name("up"));
        let down = graph.new_input(&make_name("down"));

        let count_clock = graph.new_input(&make_name("count_clock"));
        let store_clock = graph.new_input(&make_name("store_clock"));
        let clear_clock = graph.new_input(&make_name("clear_clock"));

        let ptr_count_en = graph.new_input(&make_name("ptr_count_en"));
        let data_count_en = graph.new_input(&make_name("data_count_en"));

        let reset = graph.new_input(&make_name("reset"));

        reset.connect(reg.clear());

        // Leave ram chip always enabled. We won't need it unless we want
        // to support larger ram sizes or (lol) optimize the power usage
        // TODO: is there a global low signal somewhere? should there be?
        let low = graph.new_output("LOW", Signal::Low);
        low.connect(ram.ce_inv());

        // Only the ram or the register should be outputting to the bus, not
        // both.
        //
        // During normal operation, the register output is enabled when data
        // count is enabled (i.e. we want to see the result of `+` and `-`).
        // Otherwise ram output is enabled. TODO: need to disable ram out when
        // doing an input `,` operation
        //
        // During reset, the register is outputting zero, so we want the RAM
        // to read that while cycling through the address space. This will
        // clear the contents of RAM (NYI)
        let reg_not_ram = &reset | &data_count_en;

        // Since RAM OE is inverted but the tristate enable is not, we can
        // just connect them all to the same control signal
        graph.connect_all(&[&reg_not_ram, reg_interface.en(), ram.oe_inv()]);

        // Count the data register up or down on the count clock when enabled
        let reg_count = &count_clock & &data_count_en;
        let reg_up = nand_gate(graph, "reg_up");
        let reg_down = nand_gate(graph, "reg_down");
        graph.connect_all(&[&reg_count, reg_up.input_a(), reg_down.input_a()]);
        graph.connect(&up, reg_up.input_b());
        graph.connect(&down, reg_down.input_b());
        graph.connect(reg_up.output(), reg.up());
        graph.connect(reg_down.output(), reg.down());

        // Count the pointer up or down on the count clock when enabled or during the reset clear
        //let clear = &reset & &clear_clock;
        let ptr_count = (&count_clock & &ptr_count_en) | (&reset & &clear_clock);
        let ptr_up = nand_gate(graph, "ptr_up");
        // TODO: ptr_up = nor_gate(&up & &ptr_count, &clear_ck & &reset)
        let ptr_down = nand_gate(graph, "ptr_down");
        graph.connect_all(&[&ptr_count, ptr_down.input_a(), ptr_up.input_a()]);
        graph.connect(&up, ptr_up.input_b());
        graph.connect(&down, ptr_down.input_b());
        graph.connect(ptr_up.output(), ptr.up());
        graph.connect(ptr_down.output(), ptr.down());

        // Load the reg from RAM on the store clock when ptr count is enabled (i.e. after ptr
        // crements)
        let reg_load = nand_gate(graph, "reg_load");
        graph.connect(&store_clock, reg_load.input_a());
        graph.connect(&ptr_count_en, reg_load.input_b());
        graph.connect(reg_load.output(), reg.load_inv());

        // Write from the bus to RAM on the store clock when data count is enabled (i.e. after reg
        // crements).
        let ram_we = nor_gate(graph, "ram_we");
        let write = &store_clock & &data_count_en;
        graph.connect(&reset, ram_we.input_a());
        graph.connect(&write, ram_we.input_b());
        graph.connect(ram_we.output(), ram.we_inv());

        DataBlock {
            bus,
            d_ce: data_count_en,
            p_ce: ptr_count_en,
            up,
            ptr,
            down,
            count: count_clock,
            store: store_clock,
            reset,
            clear: clear_clock,
        }
    }
}

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
