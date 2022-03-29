use befrust::*;


fn main() {
    let mut graph = Graph::default();

    let s_not = NotGate::new(&mut graph);
    let r_not = NotGate::new(&mut graph);
    let s_or = OrGate::new(&mut graph);
    let r_or = OrGate::new(&mut graph);

    // TODO: instantiate consts on the graph
    let r = graph.new_const(Signal::Low);
    let s = graph.new_const(Signal::Low);

    graph.connect(s, s_or.a);
    graph.connect(s_or.q, s_not.a);
    graph.connect(s_not.q, r_or.b);

    graph.connect(r, r_or.a);
    graph.connect(r_or.q, r_not.a);
    graph.connect(r_not.q, s_or.b);

    let q = s_not.q;

    let print = |graph: &Graph| {
        println!("{:?}",(
            graph.get_state(r),
            graph.get_state(r_or.a),
            graph.get_state(r_or.b),
            graph.get_state(r_or.q),
            graph.get_state(q),
        ));
    };

    print(&graph);
    dbg!(graph.run());
    print(&graph);

    graph.set_output(r, Signal::High);
    dbg!(graph.run());
    print(&graph);
    graph.set_output(r, Signal::Low);
    dbg!(graph.run());
    print(&graph);
    graph.set_output(s, Signal::High);
    dbg!(graph.run());
    print(&graph);
    graph.set_output(s, Signal::Low);
    dbg!(graph.run());
    print(&graph);
}
