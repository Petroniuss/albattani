extern crate kiss3d;

use kiss3d::light::Light;
use kiss3d::scene::SceneNode;
use kiss3d::window::{State, Window};
use kiss3d::nalgebra::{UnitQuaternion, Vector3};
use nalgebra::Translation3;

// ----------------------------------------------------------
//                    Open Questions
// ----------------------------------------------------------

// all right, this looks like it's exactly what I want :)

/*

API:
- load a bunch of nodes and edges
- somehow highlight the node/edge?

- we need to somehow map from simulation's node/edge to scene node.
  How do we store such a mapping?

*/


// ----------------------------------------------------------
//                    Simulation
// ----------------------------------------------------------

// might carry some additional data.
struct Edge {
    from: usize,
    to: usize
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
    fn construct_graph(
        n: usize,
        edges_iter: impl IntoIterator<Item = (usize, usize)>
    ) -> Graph {
        let mut vertices = (0..n)
            .into_iter()
            .map(|id| Vertex { id } )
            .collect();

        let mut edges = Vec::with_capacity(n);
        for _ in 0..n {
            edges.push(Vec::new());
        }

        for (from, to) in edges_iter.into_iter() {
            edges[from].push(Edge {
                from,
                to
            });
        }

        Graph {
            vertices,
            edges
        }
    }
}

trait SimulationState<'a> {
    fn graph() -> &'a Graph;
}


// ----------------------------------------------------------
//                    Rendering
// ----------------------------------------------------------

struct AppState {
    c: Vec<SceneNode>,
    rot: UnitQuaternion<f32>,
}

impl State for AppState {
    fn step(&mut self, _: &mut Window) {
        for e in self.c.iter_mut() {
            e.prepend_to_local_rotation(&self.rot)
        }
    }
}

fn main() {
    let mut window = Window::new("Kiss3d: wasm example");

    let mut nodes = vec![];
    for x in 0..10 {
        let xx = (x as f32) * 2.0;
        let mut c = window.add_cube(1.0, 1.0, 1.0);
        c.append_translation(&Translation3::new(xx, xx, xx));
        c.set_color(1.0, 0.0, 0.0);

        nodes.push(c);
    }


    window.set_light(Light::StickToCamera);

    let rot = UnitQuaternion::from_axis_angle(&Vector3::y_axis(), 0.014);
    let state = AppState { c: nodes, rot };

    window.render_loop(state)
}