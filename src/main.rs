mod core;
mod gui;

use anyhow::Result;
use log::info;

fn main() -> Result<()> {
    env_logger::init();
    info!("Starting RustSpy...");

    // Start GUI
    gui::app::run()?;

    Ok(())
}
