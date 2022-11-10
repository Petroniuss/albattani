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
- Well I'd have to read the paper first :)

*/
// ----------------------------------------------------------
//                    Implementation Notes
// ----------------------------------------------------------
/*

There are two threads:
- main thread running event loop which renders the simulation.
- simulation thread which does all the interesting computations
    that sends events via a channel to main thread.

Potentially simulation events could be recorded and then rendered offline.
*/

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
