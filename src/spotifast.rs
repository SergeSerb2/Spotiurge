//! Spotifast desktop command.

#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]

mod entrypoint;

fn main() -> eframe::Result<()> {
    // SAFETY: main has not started any worker, runtime, or platform library.
    // Consume the pairing token before any browser/visualizer can inherit it.
    unsafe { spotifast::discovery_cloud::capture_bootstrap() };
    entrypoint::run()
}
