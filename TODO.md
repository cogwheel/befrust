# TODO

## Now

Get what's already here in ship-shape.

* [ ] Finish renaming single-letter pins
  * [ ] FullAdder
  * [ ] ...
* [ ] Comprehensive tests
* [ ] Finish data block
  * [ ] Fix data counter
  * [ ] Add clear clock
* [ ] rsdoc everything
  * [ ] update links below

## Later

Make the bfpu, including any necessary or helpful updates to the compute graph

* [ ] Clocks - 3 phase: instruction -> count -> store
* [ ] Randomize RAM contents - need some re-init mechanism (chonky... might be helped by separating
  graph from execution)
* [ ] Debugging
  * [ ] Trace particular pin states
  * [ ] RunStats should have pins/nodes that are updated instead of just number of updates
  * [ ] Interactive mode (repl)

## Some day

Optimizations, generalizations, optional features, and other things that may or may not be in direct support of the bfpu
project.

* [ ] Better type safety for parts - named elements instead of vector indexes
* [ ] Consider renaming PinState (maybe Port?)
* [ ] Clean up redundant traits (`for Foo`, `for &Foo`, `for &[Foo]`, `for &[&Foo]`)
  * [ ] Use `IntoIter` instead of slices?
  * [ ] ToSignal
  * [ ] ToValue
* [ ] inline things (especially method forwards)
* [ ] only update components if their input nodes changed
* [ ] narrow: figure out a way to forward getters (and how to contact the author of derive-getters). Broad: maybe have a
  better interface for components?
* [ ] Rename `Off` so it's less confusing with `Low`
* [ ] Expose nodes as a concept - nodes can be treated as an output pin. Connecting a pin to a node merges the pin's
  existing node with the new node. This eliminates the need for an extra buffer to convert a node signal to an output
* [ ] Orthogonalize:
  * [`Signal`](src/lib.rs), [`BusValue`](src/lib.rs), and their associated traits
  * Bus gates (BusBuffer/BusTristate) and nary gates (buffer(), not_gate() etc)
* [ ] Bus values should include `Off` state
* [ ] Separate compute graph construction from execution - creating components, connecting pins, etc. should construct
  an abstract compute graph. You would build a computation engine from the abstract graph and run that. This would allow
  opportunities for optimization, injecting debug/tracing information, etc. without bogging down the graph construction
  code
* [ ] Pull states for pins
  * [ ] Consider removing bitops for signals. Meaning of `Off` depends on the pull
