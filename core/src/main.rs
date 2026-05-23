mod app;
mod applications;
mod calculator;
mod ipc;
mod platform;
mod session;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    app::run()
}
