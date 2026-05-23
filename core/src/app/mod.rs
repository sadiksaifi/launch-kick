use crate::{launcher::CoreSession, platform::paths, transport};
use std::{
    error::Error,
    io::BufReader,
    process::{Command, Stdio},
};

pub fn run() -> Result<(), Box<dyn Error>> {
    let mut ui = Command::new(paths::launcher_path())
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()?;

    let ui_stdin = ui.stdin.take().ok_or("missing UI stdin")?;
    let ui_stdout = ui.stdout.take().ok_or("missing UI stdout")?;
    let mut session = CoreSession::new();

    transport::run_ndjson_loop(BufReader::new(ui_stdout), ui_stdin, &mut session)?;

    let _ = ui.wait()?;
    Ok(())
}
