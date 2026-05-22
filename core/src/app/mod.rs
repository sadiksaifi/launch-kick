use crate::{ipc, platform::paths, session::CoreSession};
use std::{
    error::Error,
    io::{BufRead, BufReader, Write},
    process::{Command, Stdio},
};

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut ui = Command::new(paths::launcher_path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let mut ui_stdin = ui.stdin.take().ok_or("missing UI stdin")?;
    let ui_stdout = ui.stdout.take().ok_or("missing UI stdout")?;
    let mut session = CoreSession::new();

    for line in BufReader::new(ui_stdout).lines() {
        let line = line?;
        let Ok(message) = ipc::decode_client_line(&line) else {
            continue;
        };

        if let Some(response) = session.handle_client_message(message) {
            let line = ipc::encode_server_line(&response)?;
            ui_stdin.write_all(line.as_bytes())?;
            ui_stdin.flush()?;
        }
    }

    let _ = ui.wait()?;
    Ok(())
}
