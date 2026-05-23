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

    pub fn run_stdio_session<F>(executable: impl AsRef<Path>, run: F) -> io::Result<ExitStatus>
    where
        F: FnOnce(PlatformClientStdio) -> io::Result<()>,
    {
        let mut client = Self::spawn(executable)?;
        let stdio = client.take_stdio()?;

        if let Err(error) = run(stdio) {
            let _ = client.child.kill();
            let _ = client.wait();
            return Err(error);
        }

        client.wait()
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

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Read, Write};

    #[test]
    fn stdio_can_only_be_taken_once() {
        let mut client = PlatformClientProcess::spawn("/bin/cat").unwrap();
        let stdio = client.take_stdio().unwrap();

        let error = match client.take_stdio() {
            Ok(_) => panic!("stdio should only be taken once"),
            Err(error) => error,
        };
        assert_eq!(error.to_string(), "missing Platform client stdin");

        drop(stdio.stdin);
        drop(stdio.stdout);
        let _ = client.wait().unwrap();
    }

    #[test]
    fn run_stdio_session_waits_after_session_completes() {
        let status = PlatformClientProcess::run_stdio_session("/bin/cat", |mut stdio| {
            stdio.stdin.write_all(b"hello\n")?;
            drop(stdio.stdin);

            let mut output = String::new();
            stdio.stdout.read_to_string(&mut output)?;
            assert_eq!(output, "hello\n");
            Ok(())
        })
        .unwrap();

        assert!(status.success());
    }

    #[test]
    fn run_stdio_session_cleans_up_process_when_session_fails() {
        let error = PlatformClientProcess::run_stdio_session("/bin/cat", |_stdio| {
            Err(io::Error::other("transport failed"))
        })
        .unwrap_err();

        assert_eq!(error.to_string(), "transport failed");
    }
}
