mod app;
mod applications;
mod ipc;
mod launcher;
mod platform;
mod transport;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    app::run()
}
