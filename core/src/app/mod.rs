use crate::{
    launcher::CoreSession,
    platform::{client_process::PlatformClientProcess, paths},
    transport,
};
use std::error::Error;

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut session = CoreSession::new();

    PlatformClientProcess::run_stdio_session(paths::launcher_path(), |stdio| {
        transport::run_ndjson_transport(stdio.stdout, stdio.stdin, |message| {
            session.handle_client_message(message)
        })
    })?;

    Ok(())
}
