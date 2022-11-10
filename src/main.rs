extern crate kiss3d;

use color_eyre::Result;
use log::info;

use std::cell::Ref;
use std::cell::RefCell;

use std::rc::Rc;

use kiss3d::nalgebra::{UnitQuaternion, Vector3};
use kiss3d::scene::SceneNode;
use kiss3d::window::{State, Window};

use nalgebra::Point3;

use nalgebra::Rotation3;
use nalgebra::Translation3;

use color_eyre::eyre::eyre;
use rand::{thread_rng, Rng};
use random_color::{Color, Luminosity, RandomColor};
use std::sync::mpsc::{Receiver, Sender};
use std::sync::{mpsc, Arc};
use std::thread::sleep;
use std::time::Duration;
use tap::Tap;

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

type VertexId = usize;

// might carry some additional data.
#[derive(Clone, Copy, Debug)]
struct Edge {
    from: usize,
    to: usize,
}

// might carry some additional data.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
struct Vertex {
    id: VertexId,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
struct Graph {
    vertices: Vec<Vertex>,
    edges: Vec<Vec<Edge>>,
}

impl Graph {
    fn construct_graph(n: usize, edges_iter: impl IntoIterator<Item = (usize, usize)>) -> Graph {
        let vertices = (0..n).into_iter().map(|id| Vertex { id }).collect();

        let mut edges = Vec::with_capacity(n).tap_mut(|edges| {
            for _ in 0..n {
                edges.push(Vec::new());
            }
        });

        for (from, to) in edges_iter.into_iter() {
            edges[from].push(Edge { from, to });
        }

        Graph { vertices, edges }
    }
}

enum SimulationUpdate {
    EdgeSelected(Edge),
}

struct Simulation {
    graph: Graph,
    simulation_update_sender: Sender<SimulationUpdate>,
}

impl Simulation {
    fn start_simulation(&mut self) -> Result<()> {
        let mut selected_edge = self.graph.edges[0][0];

        loop {
            self.simulation_update_sender
                .send(SimulationUpdate::EdgeSelected(selected_edge));
            info!("Selected: {:?}", selected_edge);
            sleep(Duration::from_secs(2));

            let edges_num = self.graph.edges[selected_edge.to].len();
            let next_edge_idx = thread_rng().gen_range(0..edges_num);
            let next_edge = self.graph.edges[selected_edge.to][next_edge_idx];

            selected_edge = next_edge;
        }
    }
}

fn test_graph() -> Graph {
    Graph::construct_graph(5, vec![(0, 1), (1, 2), (2, 3), (3, 0), (1, 4), (4, 0)])
}

// ----------------------------------------------------------
//                    Rendering
// ----------------------------------------------------------

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

#[allow(dead_code)]
struct SimulationRenderer {
    // communication with simulation.
    simulation_update_rx: Receiver<SimulationUpdate>,

    // steady state (it rarely changes)
    vertex_nodes: Vec<Rc<RefCell<SceneNode>>>,
    edge_nodes: Vec<Vec<(VertexId, Rc<RefCell<SceneNode>>)>>,

    // temporary state
    highlighted_edge: Option<Rc<RefCell<SceneNode>>>,
}

static GRAPH_NODE_RADIUS: f32 = 0.45;
static DEFAULT_EDGE_COLOR: [f32; 3] = [0.8, 0.1, 0.2];
static HIGHLIGHT_EDGE_COLOR: [f32; 3] = [0.2, 0.8, 0.2];
static EDGE_WIDTH: f32 = 0.01;

fn gen_random_color(_node_id: usize) -> [f32; 3] {
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
        // TODO: we need to find positions for every node. somehow?
        // something sophisticated - force_graph https://docs.rs/force_graph/latest/force_graph/?
        // and something less sophisticated :)
        let mut vertex_nodes = vec![];
        for id in 0..graph.vertices.len() {
            let mut vertex_node = window.add_sphere(GRAPH_NODE_RADIUS);
            let x = id as f32;
            let y = (id % 2) as f32;
            let z = (id % 3) as f32 * 2.0;
            vertex_node.append_translation(&Translation3::new(x, y, z));

            let [r, g, b] = gen_random_color(id);
            vertex_node.set_color(r, g, b);

            vertex_nodes.push(Rc::new(RefCell::new(vertex_node)));
        }

        let mut edge_nodes = vec![];
        for _id in 0..graph.vertices.len() {
            edge_nodes.push(vec![]);
        }

        let compute_position = |node_ref: Ref<SceneNode>| {
            let position = node_ref
                .data()
                .local_translation()
                .transform_point(&Point3::origin());

            position
        };

        for edges in graph.edges.iter() {
            for &Edge { from, to } in edges {
                let from_position = compute_position(vertex_nodes[from].borrow());

                let to_position = compute_position(vertex_nodes[to].borrow());

                // render an edge.
                let edge_node = {
                    let r = EDGE_WIDTH;
                    let h = nalgebra::distance(&from_position, &to_position);

                    let mut edge_node = window.add_cylinder(r, h);

                    let color = &DEFAULT_EDGE_COLOR;
                    edge_node.set_color(color[0], color[1], color[2]);

                    let v0 = Vector3::y_axis();

                    // rotation
                    {
                        let v1 = to_position - from_position;
                        let rotation = Rotation3::rotation_between(&v0, &v1).unwrap();
                        edge_node.append_rotation(&UnitQuaternion::from(rotation));
                    }

                    // translation
                    {
                        let v1: Point3<f32> = (to_position - from_position).into();
                        edge_node.append_translation(&Translation3::from(from_position));
                        edge_node.append_translation(&Translation3::from(v1.coords / 2.0));
                    }


                    edge_node
                };

                edge_nodes[from].push((to, Rc::new(RefCell::new(edge_node))));
            }
        }

        SimulationRenderer {
            simulation_update_rx: rx,
            vertex_nodes,
            edge_nodes,
            highlighted_edge: None,
        }
    }
}

impl State for SimulationRenderer {
    fn step(&mut self, _: &mut Window) {
        if let Ok(update) = self.simulation_update_rx.try_recv() {
            match update {
                SimulationUpdate::EdgeSelected(Edge { from, to }) => {
                    if let Some(edge_node) = self.highlighted_edge.take() {
                        let color = &DEFAULT_EDGE_COLOR;
                        edge_node
                            .borrow_mut()
                            .set_color(color[0], color[1], color[2]);
                    }

                    let edge_node = self.edge_nodes[from]
                        .iter()
                        .find(|&e| e.0 == to)
                        .map(|tuple| tuple.1.clone())
                        .unwrap();

                    let color = &HIGHLIGHT_EDGE_COLOR;
                    edge_node
                        .borrow_mut()
                        .tap_mut(|e| e.set_color(color[0], color[1], color[2]));

                    self.highlighted_edge = Some(edge_node.clone());
                }
            }
        };
    }
}

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::init();

    let (tx, rx): (Sender<SimulationUpdate>, Receiver<SimulationUpdate>) = mpsc::channel();
    let graph = test_graph();

    let simulation_thread_handle = std::thread::spawn(move || {
        let mut simulation = Simulation {
            graph,
            simulation_update_sender: tx,
        };

        simulation.start_simulation()
    });

    let graph = test_graph();
    let mut window = Window::new("Kiss3d: dynamic systems simulation");
    let renderer = SimulationRenderer::from_graph(&graph, rx, &mut window);
    window.set_background_color(51.0 / 255.0, 102.0 / 255.0, 153.0 / 255.0);
    window.render_loop(renderer);

    match simulation_thread_handle.join() {
        Ok(ok) => ok,
        Err(_) => Err(eyre!("Simulation thread failed with an exception.")),
    }
}
