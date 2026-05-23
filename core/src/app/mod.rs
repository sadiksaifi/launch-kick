use crate::{
    launcher::CoreSession,
    platform::{client_process::PlatformClientProcess, paths},
    transport,
};
use std::error::Error;

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut client = PlatformClientProcess::spawn(paths::launcher_path())?;
    let stdio = client.take_stdio()?;
    let mut session = CoreSession::new();

    transport::run_ndjson_transport(stdio.stdout, stdio.stdin, |message| {
        session.handle_client_message(message)
    })?;

    let _ = client.wait()?;
    Ok(())
}
