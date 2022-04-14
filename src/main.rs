use befrust::*;
use derive_getters::Getters;
use std::fmt::{Debug, Formatter};
use std::iter::zip;

#[derive(Getters)]
pub struct DataBlock {
    d_ce: Pin,
    p_ce: Pin,
    up: Pin,
    down: Pin,
    count: Pin,
    store: Pin,
    reset: Pin,

    #[getter(skip)]
    bus: BusBuffer,

    #[getter(skip)]
    ptr: Counter16Bit,

    #[getter(skip)]
    zero: NaryGate,
}

impl Debug for DataBlock {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("DataBlock")
            .field("d", &self.d().iter().val())
            .field("a", &self.a().iter().val())
            .field("zero", &self.zero().sig())
            .finish()
    }
}

impl DataBlock {
    pub fn d(&self) -> &[Pin] {
        &self.bus.output()
    }

    pub fn zero(&self) -> &Pin {
        self.zero.output()
    }

    pub fn a(&self) -> [&Pin; 16] {
        self.ptr.output()
    }

    pub fn new(graph: &mut Graph, name: &str) -> Self {
        let make_name = |n: &str| format!("{}.{}", name, n);
        let ram = IcCY7C199::new(graph, &make_name("ram"));
        let ptr = Counter16Bit::new(graph, &make_name("ptr"));
        let reg = Counter8Bit::new(graph, &make_name("reg"));

        let buf = BusTristate::new(graph, &make_name("buf"), 8);

        let zero = nor_nary(graph, &make_name("zero"), ram.io().len());

        let bus = BusBuffer::new(graph, &make_name("bus"), 8);

        for (ram_pin, ptr_pin) in zip(ram.addr(), ptr.output()) {
            ram_pin.connect(ptr_pin);
        }

        for i in 0..8 {
            graph.connect_all(&[
                &bus.input()[i],
                &ram.io()[i],
                reg.input()[i],
                &buf.output()[i],
                &zero.input()[i],
            ]);
            graph.connect(&reg.output()[i], &buf.input()[i]);
        }

        let up = graph.new_input(&make_name("up"));
        let down = graph.new_input(&make_name("down"));

        let count = graph.new_input(&make_name("count"));
        let store = graph.new_input(&make_name("store"));

        let p_ce = graph.new_input(&make_name("p_ce"));
        let d_ce = graph.new_input(&make_name("d_ce"));

        //let mut clear_ck = graph.new_output("clear_ck", Signal::Off);
        let reset = graph.new_input(&make_name("reset"));

        let low = graph.new_output("LOW", Signal::Low);

        graph.connect_all(&[&reset, reg.clear(), ptr.clear()]);
        low.connect(ram.ce_inv());

        let reg_not_ram = &reset | &d_ce;
        graph.connect_all(&[&reg_not_ram, buf.en(), ram.oe_inv()]);

        let reg_count = &count & &d_ce;
        let reg_up = nand_gate(graph, "reg_up");
        let reg_down = nand_gate(graph, "reg_down");
        graph.connect_all(&[&reg_count, reg_up.input_a(), reg_down.input_a()]);
        graph.connect(&up, reg_up.input_b());
        graph.connect(&down, reg_down.input_b());
        graph.connect(reg_up.output(), reg.up());
        graph.connect(reg_down.output(), reg.down());

        let reg_load = nand_gate(graph, "reg_load");
        graph.connect(&store, reg_load.input_a());
        graph.connect(&d_ce, reg_load.input_b());
        graph.connect(reg_load.output(), reg.load_inv());

        let ptr_count = &count & &p_ce;
        let ptr_up = nand_gate(graph, "ptr_up");
        // TODO: ptr_up = nor_gate(&up & &ptr_count, &clear_ck & &reset)
        let ptr_down = nand_gate(graph, "ptr_down");
        graph.connect_all(&[&ptr_count, ptr_down.input_a(), ptr_up.input_a()]);
        graph.connect(&up, ptr_up.input_b());
        graph.connect(&down, ptr_down.input_b());
        graph.connect(ptr_up.output(), ptr.up());
        graph.connect(ptr_down.output(), ptr.down());

        let ram_we = nor_gate(graph, "ram_we");
        let write = &store & &d_ce;
        graph.connect(&reset, ram_we.input_a());
        graph.connect(&write, ram_we.input_b());
        graph.connect(ram_we.output(), ram.we_inv());

        DataBlock {
            zero,
            bus,
            d_ce,
            p_ce,
            up,
            ptr,
            down,
            count,
            store,
            reset,
        }
    }
}

fn main() {
    #![allow(unused_assignments, unused_mut)]

    let mut graph = Graph::new();

    let mut up = graph.new_output("up", Signal::High);
    let down = !&up;

    let mut count = graph.new_output("count", Signal::Low);
    let mut store = graph.new_output("store", Signal::Low);

    let mut p_ce = graph.new_output("p_ce", Signal::Low);
    let mut d_ce = graph.new_output("d_ce", Signal::Low);

    let d_block = DataBlock::new(&mut graph, "data");

    //let mut clear_ck = graph.new_output("clear_ck", Signal::Off);
    let mut reset = graph.new_output("reset", Signal::High);

    graph.connect_pairs(&[
        (&up, d_block.up()),
        (&down, d_block.down()),
        (&count, d_block.count()),
        (&store, d_block.store()),
        (&p_ce, d_block.p_ce()),
        (&d_ce, d_block.d_ce()),
        (&reset, d_block.reset()),
    ]);
    println!("{:?}", d_block);

    graph.run();

    // Test data reg
    println!("{:?}", d_block);

    reset.set_output(Signal::Low);
    d_ce.set_output(Signal::High);
    graph.run();
    println!("d_ce high: {:?}", d_block);

    count.set_output(Signal::High);
    graph.run();
    println!("count high: {:?}", d_block);

    count.set_output(Signal::Low);
    graph.run();
    println!("count low: {:?}", d_block);

    count.set_output(Signal::High);
    graph.run();
    println!("count high: {:?}", d_block);

    count.set_output(Signal::Low);
    graph.run();

    println!("count low: {:?}", d_block);

    // test data ptr
    d_ce.set_output(Signal::Low);
    graph.run();
    println!("d_ce low: {:?}", d_block);

    p_ce.set_output(Signal::High);
    graph.run();
    println!("p_ce high: {:?}", d_block);

    graph.pulse_output(&mut count);

    println!("ptr pulse 1{:?}", d_block);
    count.set_output(Signal::High);
    graph.run();
    count.set_output(Signal::Low);
    graph.run();
    println!("ptr pulse 2 {:?}", d_block);

    up.set_output(Signal::Low);

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

    println!("ptr down 5{:?}", d_block);

    up.set_output(Signal::High);
    graph.pulse_output(&mut count);
    graph.pulse_output(&mut count);
    graph.pulse_output(&mut count);

    println!("ptr up 3{:?}", d_block);

    p_ce.set_output(Signal::Low);
    d_ce.set_output(Signal::High);

    graph.run();

    println!("ptr up 3{:?}", d_block);
}
