# Befrust

Befrust is:

* A personal exploration with [Rust](https://www.rust-lang.org/)
* A simulation/design exercise to help keep my [bfcpu project](#my-brainfuck-computer) moving along
* A primordial form that could evolve into a general compute graph or fully-featured digital logic simulator
* Heavily inspired by [Logisim evolution](#logisim-evolution), [calc\_graph](#calc_graph), and others
* Copyright 2022 by Matthew Orlando, released under [GPL version 3](gpl-3.0.md). **TODO**: split into
    * GPL: the toplevel `bfcpu` simulation
    * LGPL: the underlying circuit graph components

Befrust can:

* Model digital circuits as a collection of parts with pins connected by nodes
* Handle cycles with tick-based propagation between parts and nodes
* Represent non-binary signals: Off (disconnected), Low, High, Error (invalid/unknown)
* Dynamically changing pin types (input/output/highZ)

Befrust doesn't:

* Care about wall time - propagation delay is one tick for every component in the graph
* Care about power (vcc, ground, or anything remotely analog)
* Expect components to be removed or pins to be disconnected

Befrust will have:

* Busses, i.e. multiple signals packed together as a group
* Conversions between signals/busses and numbers
* Better QoL. The interface of the graph, the creation of custom parts, etc. will be cleaned up to make it convenient, comprehensible, and more statically safe


## TODO: rustdoc


# Inspiration

## My Brainfuck computer

https://github.com/cogwheel/bfpu

I need to finish designing the control circuitry, particularly choosing a looping mechanism.

### Linear

When jumping, step through and ignore one instruction each cycle until reaching the jump target.

Pros: simple - state is just a counter that starts reset to 0

Cons: slow - constant time execution of each instruction (including jumps) was one of the early design goals

### Preprocessed

During reset (while RAM is being zeroed\*), create a cache of jump targets given a program pointer.

Pros:

* constant time - jump is just "load program pointer".
* cohesive - separates loop detection from loop execution

Cons:

* another RAM chip with all the interface circuitry to go along with it
* complicated reset process - Feels more like a compiler than a CPU

-----

\* Zeroing ram also has a "feels more like a compiler than a CPU" aspect. This could be amortized instead: use a counter to keep track of visited RAM locations and reset the data register instead of loading from RAM. However, the current reset approach is just "connect some fast clocks to existing circuitry"

### Amortized

Have a counter representing visited program pointer locations, and a RAM indexed by the program pointer. When encountering a loop instruction for the first time, step through each instruction linearly, building the cache along the way. When encountring a loop instruction that has been seen before, jump the same way as in [Preprocessed](#preprocessed)

Pros: more like the branch prediction logic in a real CPU

Cons:

* all the extra components and circuitry of the Preprocessed version, plus:
  * an aditional counter
  * interwoven logic (hence circuitry) between cache generation and jump execution

So yeah, being able to rapidly change the circuit layout and test different variations of components, optimizations, etc. was the main driving force.

## Logisim Evolution

https://github.com/logisim-evolution/logisim-evolution

I've implemented a lot of this in logisim-evolution, including custom 74193 chips. I got to the point where I need to design the looping and I/O circuitry and have hit a bit of a wall.

The GUI is getting in the way of making rapid, simple changes. In code, swapping two connections is as simple as changing the names of two variables. In logisim it's almost a drawing minigame where you have to be careful not to unintentionally connect things, accidentally move things because of a misclick, delete the wrong thing because "finger" doesn't select it and the selection handles are the same color as the background, etc.

However, I absolutely love the abstract model it uses to represent circuits. There are components which have pins, and you connect the pins with wires. Every tick, the components update their output pins given the state of the wires connected to their input pins, and then the state of the output pins is transferred onto the wires.

Befrust uses the same model, albeit with different names. You construct a number of components, create connections between their pins, and run the various propagation functions.

## calc_graph

https://github.com/1tgr/rust-calc-graph

I initially tried to implement the bfpu using the `calc_graph` library. This helped me grease my wheels with Rust since it's not my primary language. However I ran into a few limitations.

Pins and components are naturally modeled as sources and mappings. You create some input sources, then use map, zip, zip3, etc. to assign those inputs to a lambda. The return value of the lambda becomes a new source.

* depends on build-time code generation
    * separate concrete zip_N_ functions for every number of inputs up to 8
    * no code completion for these generated functions in IntelliJ
* can't directly represent pins changing from inputs to outputs
* a component with multiple outputs needs to be represented as either:
    * a single output object, which means you can't use elements of the object as signal sources directly
    * a separate call to `map/zip` for every "signal" in the output
* had to build an extra layer of macros to make an interface to wrangle boilerplate
* combinatorial explosion of compilation time as components were added. Resolved by adding a bunch of `.boxed()`s everywhere, leading to even more boilerplate

The first point is a deal-breaker on its own; there is no way to create a component with more than 8 inputs without altering the compilation arguments for a 3rd-party dependency.

Then, with the mismatch between the computation model I imagined and the one `calc_graph` provided, I had written about as much code to wrap `calc_graph` as I've written to implement my own graph in Befrust.

To be clear, this is no dig on calc_graph. This is just a case of picking the wrong tool for the job.
