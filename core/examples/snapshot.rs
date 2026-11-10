//! Prints what Velapor sees on this machine as JSON, without the window.
//!
//! `cargo run -p velapor-core --example snapshot`

use velapor_core::Watchman;

/// Takes two beats (the processor load needs one to settle) and prints the second.
fn main() {
    let mut watchman = Watchman::new();
    watchman.tick();
    std::thread::sleep(Watchman::BEAT);
    let tick = watchman.tick();
    match serde_json::to_string_pretty(&tick.snapshot) {
        Ok(json) => println!("{json}"),
        Err(error) => eprintln!("The snapshot could not be written as JSON: {error}"),
    }
}
