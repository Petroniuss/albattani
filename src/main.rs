extern crate kiss3d;

use kiss3d::light::Light;
use kiss3d::nalgebra::{UnitQuaternion, Vector3};
use kiss3d::scene::SceneNode;
use kiss3d::window::{State, Window};
use nalgebra::Translation3;
use random_color::{Color, Luminosity, RandomColor};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

// ----------------------------------------------------------
//                    Open Questions
// ----------------------------------------------------------
/*

We want to be able to simulate walking through a graph:
    So we may want to create a simple simulation which just follows
    some random nodes connected by edges and highlight a node an edge that was selected.

    For that we need to be able to draw a graph
    - somehow place the nodes in some random positions
        - we may want to run force simulation there.
    - connect the nodes via edges
    - highlight a node and an edge that was selected by a simulation.
    - make sure that simulation moves smoothly.

Simulation API:
- load a bunch of nodes and edges - check.


- we need to somehow map from simulation's node/edge to scene node.
    How do we store such a mapping?

Rendering API:
- somehow highlight the node/edge?
    This has to be indicated by simulation I guess.

*/
// ----------------------------------------------------------
//                    Simulation
// ----------------------------------------------------------

// might carry some additional data.
struct Edge {
    from: usize,
    to: usize,
}

// might carry some additional data.
struct Vertex {
    id: usize,
}

struct Graph {
    vertices: Vec<Vertex>,
    edges: Vec<Vec<Edge>>,
}

impl Graph {
    fn construct_graph(n: usize, edges_iter: impl IntoIterator<Item = (usize, usize)>) -> Graph {
        let mut vertices = (0..n).into_iter().map(|id| Vertex { id }).collect();

        let mut edges = Vec::with_capacity(n);
        for _ in 0..n {
            edges.push(Vec::new());
        }

        for (from, to) in edges_iter.into_iter() {
            edges[from].push(Edge { from, to });
        }

        Graph { vertices, edges }
    }
}

trait SimulationState<'a> {
    fn graph() -> &'a Graph;
}

type VertexId = usize;

enum SimulationUpdate {
    EdgeSelected(Edge),
}

trait Simulation {
    fn iteration() -> SimulationUpdate {
        // modify its internal state
        // and output some change so that the UI layer knows
        todo!();
    }
}

fn test_graph() -> Graph {
    Graph::construct_graph(5, vec![(0, 1), (1, 2), (2, 3), (3, 0), (1, 4), (4, 0)])
}

// ----------------------------------------------------------
//                    Rendering
// ----------------------------------------------------------

trait Renderer {
    fn show_update(update: SimulationUpdate) {}
}

// we can have some main loop of the simulation.
// and we need two way communication.
// Simulation -> iteration() -> show_update() -> onComplete() -> loop!
// we need some sort of a callback.

// or we could use channels:
// in the event loop check if there's something in the channel
// if so start rendering it
// once done (after several frames..)
// send a message through a different channel to the simulation
// that rendering of a previous step has completed.
// the simulation thread can simpy sleep waiting on a channel.
// while the renderer might have to poll the queue or something.
// sounds cool!

struct SimulationRenderer {
    // communication with simulation.
    simulation_update_rx: Receiver<SimulationUpdate>,

    // steady state (it rarely changes)
    graph_nodes: Vec<SceneNode>, // todo: we need to figure out a way how to draw edges!

                                 // temporary state. (might change on every frame I guess)
                                 // ...
}

static GRAPH_NODE_RADIUS: f32 = 0.45;

fn gen_random_color(node_id: usize) -> [f32; 3] {
    let [r, g, b] = RandomColor::new()
        .hue(Color::Purple) // Optional
        .luminosity(Luminosity::Light)
        .alpha(1.0) // Optional
        .to_rgb_array();

    let norm = |v: u8| v as f32 / 255.0;
    [norm(r), norm(g), norm(b)]
}

impl SimulationRenderer {
    fn from_graph(
        graph: &Graph,
        rx: Receiver<SimulationUpdate>,
        window: &mut Window,
    ) -> SimulationRenderer {
        // TODO: here we may want to find positions for every node. somehow?
        // force_graph: https://docs.rs/force_graph/latest/force_graph/
        let mut graph_nodes = vec![];

        for id in 0..graph.vertices.len() {
            let mut graph_node = window.add_sphere(GRAPH_NODE_RADIUS);
            let x = id as f32;
            let y = (id % 2) as f32;
            let z = (id % 3) as f32 * 2.0;
            graph_node.append_translation(&Translation3::new(x, y, z));

            let [r, g, b] = gen_random_color(id);
            graph_node.set_color(r, g, b);

            graph_nodes.push(graph_node);
        }

        SimulationRenderer {
            simulation_update_rx: rx,
            graph_nodes,
        }
    }
}

impl State for SimulationRenderer {
    fn step(&mut self, _: &mut Window) {
        // TODO() -> check if there's something that we have to do?
    }
}

fn main() {
    let mut window = Window::new("Kiss3d: dynamic systems simulation");
    let (tx, rx): (Sender<SimulationUpdate>, Receiver<SimulationUpdate>) = mpsc::channel();
    let graph = test_graph();
    let renderer = SimulationRenderer::from_graph(&graph, rx, &mut window);

    window.render_loop(renderer)
}
