use std::{
    io,
    path::Path,
    process::{Child, ChildStdin, ChildStdout, Command, ExitStatus, Stdio},
};

pub struct PlatformClientProcess {
    child: Child,
    stdin: Option<ChildStdin>,
    stdout: Option<ChildStdout>,
}

pub struct PlatformClientStdio {
    pub stdin: ChildStdin,
    pub stdout: ChildStdout,
}

impl PlatformClientProcess {
    pub fn spawn(executable: impl AsRef<Path>) -> io::Result<Self> {
        let mut child = Command::new(executable.as_ref())
            .stdin(Stdio::piped())
            .stdout(Stdio::piped())
            .stderr(Stdio::inherit())
            .spawn()?;

        Ok(Self {
            stdin: child.stdin.take(),
            stdout: child.stdout.take(),
            child,
        })
    }

    pub fn take_stdio(&mut self) -> io::Result<PlatformClientStdio> {
        let stdin = self
            .stdin
            .take()
            .ok_or_else(|| io::Error::other("missing Platform client stdin"))?;
        let stdout = self
            .stdout
            .take()
            .ok_or_else(|| io::Error::other("missing Platform client stdout"))?;

        Ok(PlatformClientStdio { stdin, stdout })
    }

    pub fn wait(&mut self) -> io::Result<ExitStatus> {
        self.child.wait()
    }
}
