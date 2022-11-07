extern crate kiss3d;

use kiss3d::light::Light;
use kiss3d::nalgebra::{UnitQuaternion, Vector3};
use kiss3d::scene::SceneNode;
use kiss3d::window::{State, Window};
use nalgebra::{Isometry3, Point3, Quaternion, Rotation3, Translation3};
use random_color::{Color, Luminosity, RandomColor};
use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};
use kiss3d::ncollide3d::query::NeighborhoodGeometry::Point;

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
    // Graph::construct_graph(2, vec![
    //     (0, 1)]
    // )
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
    vertex_nodes: Vec<SceneNode>, // todo: we need to figure out a way how to draw edges!

                                 // temporary state. (might change on every frame I guess)
                                 // ...
}

static GRAPH_NODE_RADIUS: f32 = 0.45;
static EDGE_COLOR: [f32; 3] = [0.8, 0.1, 0.2];
static EDGE_WIDTH: f32 = 0.01;

fn gen_random_color(node_id: usize) -> [f32; 3] {
    let [r, g, b] = RandomColor::new()
        .hue(Color::Purple) // Optional
        .luminosity(Luminosity::Light)
        .alpha(1.0) // Optional
        .to_rgb_array();

    let norm = |v: u8| v as f32 / 255.0;
    [norm(r), norm(g), norm(b)]
}

// TODO: render graph edges.
impl SimulationRenderer {
    fn from_graph(
        graph: &Graph,
        rx: Receiver<SimulationUpdate>,
        window: &mut Window,
    ) -> SimulationRenderer {
        // TODO: we need to find positions for every node. somehow?
        // something sophisticated - force_graph https://docs.rs/force_graph/latest/force_graph/?
        let mut vertex_nodes = vec![];


        // and something less sophisticated :)
        for id in 0..graph.vertices.len() {
            let mut vertex_node = window.add_sphere(GRAPH_NODE_RADIUS);
            let x = id as f32;
            let y = (id % 2) as f32;
            let z = (id % 3) as f32 * 2.0;
            vertex_node.append_translation(&Translation3::new(x, y, z));

            let [r, g, b] = gen_random_color(id);
            vertex_node.set_color(r, g, b);

            vertex_nodes.push(vertex_node);
        }

        // add obj -> path?
        // generate a path based on a node's position.
        use crate::Edge;

        // well this needs to happen inside simulation renderer.
        // well these lines should really be some sort of scene nodes.
        for edges in graph.edges.iter() {
            for &Edge { from, to } in edges {
                let from_node = &vertex_nodes[from];
                let to_node = &vertex_nodes[to];


                let from_position = from_node
                    .data()
                    .local_translation()
                    .transform_point(&Point3::origin());

                let to_position = to_node
                    .data()
                    .local_translation()
                    .transform_point(&Point3::origin());

                // well this could be our line:
                {
                    let r = EDGE_WIDTH;
                    let h = nalgebra::distance(&from_position, &to_position);

                    let mut edge_node = window.add_cylinder(r, h);
                    edge_node.set_color(EDGE_COLOR[0], EDGE_COLOR[1], EDGE_COLOR[2]);

                    // first apply rotation!
                    // and then apply translation!

                    // translation
                    // {
                    //     edge_node.append_translation(&Translation3::from(from_position));
                    //     edge_node.append_translation(&Translation3::from([0.0, h / 2.0, 0.0]));
                    // }

                    // rotation
                    {
                        // that's the tricky bit I guess.
                        // well that's a little bit of math . ;d

                        let direction = to_position.coords - from_position.coords;
                        // let up = Vector3::y();
                        //
                        // // I am not sure if this should be position
                        let rotation = Rotation3::rotation_between(
                            &edge_node.data().local_translation().vector,
                            // edge_node.data().local_rotation().to_rotation_matrix(),
                            &Vector3::from_data(direction.data),
                        ).unwrap_or_else(|| Rotation3::identity());

                        let rotation: Rotation3<f32> = Rotation3::rotation_between(
                            &Vector3::y(),
                            &Vector3::x()
                        ).unwrap();


                        // println!("from_node: {}, to_node: {}", from, to);
                        // println!("from_position: {}", from_position);
                        // println!("to_position: {}", to_position);
                        // println!("direction: {}", direction);
                        //
                        // println!("rotation:{}", rotation);
                        //
                        // println!("initial_rotation: {}", edge_node.data().local_rotation());
                        // rotations are with respect to the center,
                        // not with the respect to the beginning of the cylinder
                        // how to deal with this?
                        let cylinder_position = Point3::from(edge_node.data().local_translation().vector);
                        println!("cylinder_position: {}", cylinder_position);


                        edge_node.append_rotation(&UnitQuaternion::from(rotation));
                    }
                }

                {
                    let r = EDGE_WIDTH;
                    let h = nalgebra::distance(&from_position, &to_position);

                    let mut edge_node = window.add_cylinder(r, h);
                    edge_node.set_color(EDGE_COLOR[0], EDGE_COLOR[1], EDGE_COLOR[2]);


                    // translation
                    {
                        edge_node.append_translation(&Translation3::from(from_position));
                        edge_node.append_translation(&Translation3::from([0.0, h / 2.0, 0.0]));
                    }
                }

            }
        }


        SimulationRenderer {
            simulation_update_rx: rx,
            vertex_nodes,
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
    window.set_background_color(51.0 / 255.0, 102.0 / 255.0, 153.0 / 255.0);

    window.render_loop(renderer)
}
