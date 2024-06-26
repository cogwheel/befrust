use crate::*;

use std::fmt::{Debug, Formatter};
use std::iter::zip;

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
