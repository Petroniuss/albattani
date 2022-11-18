use std::sync::mpsc::Sender;
use std::thread::sleep;
use std::time::Duration;

use color_eyre::Result;
use log::info;
use rand::{thread_rng, Rng};
use tap::Tap;

// ----------------------------------------------------------
//                    Simulation
// ----------------------------------------------------------

/// SimulationRenderer should react to these events
/// and render the simulation accordingly.
pub(crate) enum SimulationUpdate {
    HighlightEdge(Edge),
}

pub(crate) trait Simulation {
    /// emits SimulationUpdate events.
    fn start_simulation(self, simulation_parameters: SimulationParameters) -> Result<()>;
}

pub(crate) struct SimpleSimulation;

pub(crate) struct SimulationParameters {
    pub(crate) graph: Graph,
    pub(crate) simulation_update_sender: Sender<SimulationUpdate>,
}

impl Simulation for SimpleSimulation {
    fn start_simulation(self, simulation_parameters: SimulationParameters) -> Result<()> {
        let SimulationParameters {
            graph,
            simulation_update_sender,
        } = simulation_parameters;

        let mut simulation_state = SimulationState {
            graph,
            simulation_update_sender,
        };

        simulation_state.start_simulation()
    }
}

struct SimulationState {
    graph: Graph,
    simulation_update_sender: Sender<SimulationUpdate>,
}

impl SimulationState {
    pub(crate) fn start_simulation(&mut self) -> Result<()> {
        let mut selected_edge = self.graph.edges[0][0];

        loop {
            if let Err(_err) = self
                .simulation_update_sender
                .send(SimulationUpdate::HighlightEdge(selected_edge))
            {
                return Ok(());
            }

            info!("Highlighted: {:?}", selected_edge);
            sleep(Duration::from_secs(2));

            let edges_num = self.graph.edges[selected_edge.to].len();
            let next_edge_idx = thread_rng().gen_range(0..edges_num);
            let next_edge = self.graph.edges[selected_edge.to][next_edge_idx];

            selected_edge = next_edge;
        }
    }
}

pub(crate) type VertexId = usize;

// might carry some additional data.
#[derive(Clone, Copy, Debug)]
pub(crate) struct Edge {
    pub(crate) from: VertexId,
    pub(crate) to: VertexId,
}

// might carry some additional data.
#[derive(Clone, Copy, Debug)]
#[allow(dead_code)]
pub(crate) struct Vertex {
    id: VertexId,
}

#[allow(dead_code)]
#[derive(Clone, Debug)]
pub(crate) struct Graph {
    pub(crate) vertices: Vec<Vertex>,
    pub(crate) edges: Vec<Vec<Edge>>,
}

impl Graph {
    /// Just a utility, we're probably going to load a graph from a file.
    /// Might be useful for tests or something.
    pub(crate) fn construct_graph(
        n: usize,
        edges_iter: impl IntoIterator<Item = (VertexId, VertexId)>,
    ) -> Graph {
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
