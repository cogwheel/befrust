# TODO

## Now

Get what's already here in ship-shape.

* [X] Finish renaming single-letter pins
  * [X] FullAdder
* [X] rsdoc everything
  * [X] add doc comments
  * [X] probably get rid of derive-getters
* [ ] Comprehensive tests
  * [ ] Why did tests succeed when ic74193's `output()` returned `dN*` instead of `qN*`?
* [X] Finish data block
  * [X] Fix data counter (works, but slow; cheating for now)
  * [X] Add clear clock

## Later

Make the bfpu, including any necessary or helpful updates to the compute graph

* [ ] publish the docs
  * [ ] maybe need to split this into front/back-end
  * [ ] update links below
* [ ] Helper to connect busses (like connect\_many, NaryGate::connect\_inputs())
* [ ] Clocks - 3 phase: instruction -> count -> store
* [ ] Randomize RAM contents - need some re-init mechanism (chonky... might be helped by separating
  graph from execution)
* [ ] Debugging
  * [ ] more custom Debug implementations
  * [ ] Trace particular pin states
  * [ ] RunStats should have pins/nodes that are updated instead of just number of updates
  * [ ] Interactive mode (repl)
* [ ] run\_for() - cap number of ticks instead of using hash set

## Some day

Optimizations, generalizations, optional features, and other things that may or may not be in direct support of the bfpu
project.

* [ ] Better type safety for parts - named elements instead of vector indexes
* [ ] Consider renaming PinState (maybe Port?)
* [ ] Consider removing bitops for PinState
* [ ] Clean up redundant traits (`for Foo`, `for &Foo`, `for &[Foo]`, `for &[&Foo]`)
  * [ ] Use `IntoIter` instead of slices?
  * [ ] ToSignal
  * [ ] ToValue
* [ ] inline things (especially method forwards)
* [ ] look into alternative to `either_are`, `both_are`, `one_is`, etc
* [ ] only update components if their input nodes changed
* [ ] Rename `Off` so it's less confusing with `Low`
* [ ] Expose nodes as a concept - nodes can be treated as an output pin. Connecting a pin to a node merges the pin's
  existing node with the new node. This eliminates the need for an extra buffer to convert a node signal to an output
* [ ] Orthogonalize:
  * [`Signal`](src/lib.rs), [`BusValue`](src/lib.rs), and their associated traits
  * Bus gates (BusBuffer/BusTristate) and nary gates (buffer(), not\_gate() etc)
* [ ] Bus values should include `Off` state
* [ ] Separate compute graph construction from execution - creating components, connecting pins, etc. should construct
  an abstract compute graph. You would build a computation engine from the abstract graph and run that. This would allow
  opportunities for optimization, injecting debug/tracing information, etc. without bogging down the graph construction
  code
* [ ] Pull states for pins
  * [ ] Consider removing bitops for signals. Meaning of `Off` depends on the pull
