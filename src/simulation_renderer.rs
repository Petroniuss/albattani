use std::cell::Ref;
use std::cell::RefCell;
use std::rc::Rc;

use kiss3d::nalgebra::{UnitQuaternion, Vector3};
use kiss3d::scene::SceneNode;

use nalgebra::Point3;
use nalgebra::Rotation3;
use nalgebra::Translation3;

use random_color::{Color, Luminosity, RandomColor};

use crate::simulation::{Edge, Graph, SimulationUpdate, VertexId};
use colors_transform::{Color as RgbColor, Rgb};
use kiss3d::window::{State, Window};
use lazy_static::lazy_static;
use std::sync::mpsc::Receiver;
use tap::Tap;

#[allow(dead_code)]
pub(crate) struct SimulationRenderer {
    /// communication with simulation.
    simulation_update_rx: Receiver<SimulationUpdate>,

    /// steady state (it rarely changes)
    vertex_nodes: Vec<Rc<RefCell<SceneNode>>>,
    edge_nodes: Vec<Vec<(VertexId, Rc<RefCell<SceneNode>>)>>,

    /// temporary state
    highlighted_edge: Option<Rc<RefCell<SceneNode>>>,
}

static GRAPH_NODE_RADIUS: f32 = 0.45;
static EDGE_WIDTH: f32 = 0.01;

lazy_static! {
    static ref HIGHLIGHT_EDGE_COLOR: Rgb = Rgb::from_hex_str("e63946").unwrap();
    static ref DEFAULT_EDGE_COLOR: Rgb = Rgb::from_hex_str("a7c957").unwrap();
}

fn set_node_color(node: &mut SceneNode, color: &Rgb) {
    node.set_color(
        color.get_red() / 255.0,
        color.get_green() / 255.0,
        color.get_blue() / 255.0,
    );
}

fn gen_random_color(_node_id: usize) -> [f32; 3] {
    let [r, g, b] = RandomColor::new()
        .hue(Color::Orange)
        .luminosity(Luminosity::Light)
        .alpha(1.0)
        .to_rgb_array();

    let norm = |v: u8| v as f32 / 255.0;
    [norm(r), norm(g), norm(b)]
}

impl State for SimulationRenderer {
    fn step(&mut self, _: &mut Window) {
        if let Ok(update) = self.simulation_update_rx.try_recv() {
            match update {
                SimulationUpdate::HighlightEdge(Edge { from, to }) => {
                    if let Some(edge_node) = self.highlighted_edge.take() {
                        edge_node
                            .borrow_mut()
                            .tap_mut(|node| set_node_color(node, &DEFAULT_EDGE_COLOR));
                    }

                    let edge_node = self.edge_nodes[from]
                        .iter()
                        .find(|&e| e.0 == to)
                        .map(|tuple| tuple.1.clone())
                        .unwrap();

                    edge_node
                        .borrow_mut()
                        .tap_mut(|node| set_node_color(node, &HIGHLIGHT_EDGE_COLOR));

                    self.highlighted_edge = Some(edge_node.clone());
                }
            }
        };
    }
}

impl SimulationRenderer {
    pub(crate) fn from_graph(
        graph: &Graph,
        rx: Receiver<SimulationUpdate>,
        window: &mut Window,
    ) -> SimulationRenderer {
        // TODO: we need to find positions for every node. somehow?
        // something sophisticated - force_graph https://docs.rs/force_graph/latest/force_graph/?
        // and here's something less sophisticated :)
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
                    set_node_color(&mut edge_node, &DEFAULT_EDGE_COLOR);

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
