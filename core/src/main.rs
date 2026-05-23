mod app;
mod applications;
mod ipc;
mod platform;
mod session;

use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    app::run()
}
