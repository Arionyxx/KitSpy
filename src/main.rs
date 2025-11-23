mod core;
mod gui;

use anyhow::Result;
use log::info;

fn main() -> Result<()> {
    env_logger::init();
    info!("Starting RustSpy...");

    // Example usage of core
    let loader = core::loader::Loader::new();
    loader.load("test_path")?;

    // Start GUI
    gui::app::run()?;

    Ok(())
}
