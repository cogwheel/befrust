use befrust::*;
use derive_getters::Getters;
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
    bus: Vec<Pin>,

    #[getter(skip)]
    ptr: Counter16Bit,

    #[getter(skip)]
    zero: NaryGate,
}

impl DataBlock {
    pub fn d(&self) -> &[Pin] {
        &self.bus[8..16]
    }

    pub fn zero(&self) -> &Pin {
        self.zero.q()
    }

    pub fn a(&self) -> [&Pin; 16] {
        self.ptr.q()
    }

    pub fn new(graph: &mut Graph, name: &str) -> Self {
        let make_name = |n: &str| format!("{}.{}", name, n);
        let ram = IcCY7C199::new(graph, &make_name("ram"));
        let ptr = Counter16Bit::new(graph, &make_name("ptr"));
        let reg = Counter8Bit::new(graph, &make_name("reg"));

        let buf = TristateBuffer::new(graph, &make_name("buf"), 8);

        let zero = nor_nary(graph, &make_name("zero"), ram.d().len());

        let mut bus_states = vec![PinState::INPUT; 16];
        bus_states[8..16].fill(PinState::OUTPUT);
        let bus = graph.new_part(&make_name("bus"), &bus_states, |before, after| {
            for i in 0..8 {
                after[i + 8] = match before[i] {
                    PinState::Input(s) => PinState::Output(s),
                    _ => panic!("Unexpected pin state"),
                }
            }
        });

        for (ram_pin, ptr_pin) in zip(ram.a(), ptr.d()) {
            ram_pin.connect(ptr_pin);
        }

        for i in 0..8 {
            graph.connect_all(&[&bus[i], &ram.d()[i], reg.d()[i], &buf.q()[i], &zero.a()[i]]);
            graph.connect(&reg.q()[i], &buf.a()[i]);
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
        let reg_down = nand_gate(graph, "reg_up");
        graph.connect_all(&[&reg_count, reg_up.a(), reg_down.a()]);
        graph.connect(&up, reg_up.b());
        graph.connect(&down, reg_down.b());
        graph.connect(reg_up.q(), reg.up());
        graph.connect(reg_down.q(), reg.down());

        let reg_load = nand_gate(graph, "reg_load");
        graph.connect(&store, reg_load.a());
        graph.connect(&d_ce, reg_load.b());
        graph.connect(reg_load.q(), reg.load_inv());

        let ptr_count = &count & &p_ce;
        let ptr_up = nand_gate(graph, "ptr_up");
        // TODO: ptr_up = nor_gate(&up & &ptr_count, &clear_ck & &reset)
        let ptr_down = nand_gate(graph, "ptr_down");
        graph.connect_all(&[&ptr_count, ptr_down.a(), ptr_up.a()]);
        graph.connect(&up, ptr_up.b());
        graph.connect(&down, ptr_down.b());
        graph.connect(ptr_up.q(), ptr.up());
        graph.connect(ptr_down.q(), ptr.down());

        let ram_we = nor_gate(graph, "ram_we");
        let write = &store & &d_ce;
        graph.connect(&reset, ram_we.a());
        graph.connect(&write, ram_we.b());
        graph.connect(ram_we.q(), ram.we_inv());

        // TODO: there must be a cleaner way to turn slice of references into slice of owned clones
        let mut a = vec![];
        for pin in ptr.q() {
            a.push(pin.clone())
        }

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

// TODO: make brainfuck computer
fn main() {
    let mut graph = Graph::new();

    let mut up = graph.new_output("up", Signal::High);
    let down = !&up;

    let mut count = graph.new_output("count", Signal::Low);
    let mut store = graph.new_output("store", Signal::Low);

    let mut p_ce = graph.new_output("p_ce", Signal::Low);
    let mut d_ce = graph.new_output("d_ce", Signal::Low);

    //let mut clear_ck = graph.new_output("clear_ck", Signal::Off);
    let mut reset = graph.new_output("reset", Signal::High);

    let d_block = DataBlock::new(&mut graph, "data");

    graph.connect_pairs(&[
        (&up, d_block.up()),
        (&down, d_block.down()),
        (&count, d_block.count()),
        (&store, d_block.store()),
        (&p_ce, d_block.p_ce()),
        (&d_ce, d_block.d_ce()),
        (&reset, d_block.reset()),
    ]);
    println!("reg {:?}", d_block.d().iter().val());

    dbg!(graph.run());

    // Test data reg
    println!("reg {:?}", d_block.d().iter().val());

    reset.set_output(Signal::Low);
    d_ce.set_output(Signal::High);
    count.set_output(Signal::High);

    graph.run();

    println!("reg {:?}", d_block.d().iter().val());

    count.set_output(Signal::Low);
    graph.run();
    count.set_output(Signal::High);
    graph.run();
    count.set_output(Signal::Low);
    graph.run();

    println!("reg {:?}", d_block.d().iter().val());

    // test data ptr
    println!("ptr {:?}", d_block.a().iter().val());

    d_ce.set_output(Signal::Low);
    p_ce.set_output(Signal::High);

    count.set_output(Signal::High);
    graph.run();
    count.set_output(Signal::Low);
    graph.run();

    println!("ptr {:?}", d_block.a().iter().val());
}
