mod simulation;
mod simulation_renderer;

use std::sync::mpsc;
use std::sync::mpsc::{Receiver, Sender};

use kiss3d::window::Window;

use color_eyre::eyre::eyre;
use color_eyre::Result;
use colors_transform::{Color, Rgb};
use log::LevelFilter;

use crate::simulation::{
    Graph, SimpleSimulation, Simulation, SimulationParameters, SimulationUpdate,
};
use crate::simulation_renderer::SimulationRenderer;

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

fn test_graph() -> Graph {
    Graph::construct_graph(5, vec![(0, 1), (1, 2), (2, 3), (3, 0), (1, 4), (4, 0)])
}

fn main() -> Result<()> {
    color_eyre::install()?;
    env_logger::builder()
        .filter_level(LevelFilter::Info)
        .try_init()?;

    let (tx, rx): (Sender<SimulationUpdate>, Receiver<SimulationUpdate>) = mpsc::channel();
    let graph = test_graph();

    let simulation_thread_handle = std::thread::spawn(move || {
        let simulation_parameters = SimulationParameters {
            graph,
            simulation_update_sender: tx,
        };

        SimpleSimulation.start_simulation(simulation_parameters)
    });

    let graph = test_graph();
    let mut window = initialize_window()?;
    let simulation_renderer = SimulationRenderer::from_graph(&graph, rx, &mut window);
    window.render_loop(simulation_renderer);

    match simulation_thread_handle.join() {
        Ok(ok) => ok,
        Err(_) => Err(eyre!("Simulation thread failed with an exception.")),
    }
}

fn initialize_window() -> Result<Window> {
    let mut window = Window::new("Kiss3d: dynamic systems simulation");

    let background_color =
        Rgb::from_hex_str("023e8a").map_err(|e| eyre!("hex string parsing failure: {:#?}", e))?;

    window.set_background_color(
        background_color.get_red() / 255.0,
        background_color.get_green() / 255.0,
        background_color.get_blue() / 255.0,
    );

    Ok(window)
}
