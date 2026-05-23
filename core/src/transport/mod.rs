use crate::{ipc, launcher::CoreSession};
use std::io::{self, BufRead, Write};

pub fn run_ndjson_loop<R, W>(reader: R, mut writer: W, session: &mut CoreSession) -> io::Result<()>
where
    R: BufRead,
    W: Write,
{
    for line in reader.lines() {
        let line = line?;
        let Ok(message) = ipc::decode_client_line(&line) else {
            continue;
        };

        for response in session.handle_client_message(message) {
            let line = ipc::encode_server_line(&response).map_err(io::Error::other)?;
            writer.write_all(line.as_bytes())?;
            writer.flush()?;
        }
    }

    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::applications::Applications;
    use serde_json::Value;
    use std::{
        fs,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[cfg(unix)]
    use std::{io::ErrorKind, os::unix::process::ExitStatusExt, process::ExitStatus};

    #[test]
    fn eof_exits_cleanly() {
        let mut session = test_session(Vec::new(), true);
        let mut output = Vec::new();

        run_ndjson_loop(io::Cursor::new(Vec::<u8>::new()), &mut output, &mut session).unwrap();

        assert!(output.is_empty());
    }

    #[test]
    fn malformed_lines_are_ignored_and_loop_continues() {
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("Safari.app/Contents")).unwrap();
        let mut session = test_session(vec![root.clone()], true);
        let input = b"not json\n{\"type\":\"app::list\"}\n";
        let mut output = Vec::new();

        run_ndjson_loop(io::Cursor::new(input), &mut output, &mut session).unwrap();

        let lines = output_lines(&output);
        assert_eq!(lines.len(), 1);
        assert_eq!(json_type(lines[0]), "app::list");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn multiple_messages_produce_multiple_server_lines() {
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("Safari.app/Contents")).unwrap();
        let mut session = test_session(vec![root.clone()], true);
        let input = b"{\"type\":\"app::list\"}\n{\"type\":\"app::launch\",\"path\":\"/Applications/Safari.app\"}\n";
        let mut output = Vec::new();

        run_ndjson_loop(io::Cursor::new(input), &mut output, &mut session).unwrap();

        let lines = output_lines(&output);
        assert_eq!(lines.len(), 2);
        assert_eq!(json_type(lines[0]), "app::list");
        assert_eq!(json_type(lines[1]), "app::launch::result");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn flushes_after_each_server_message() {
        let mut session = test_session(Vec::new(), true);
        let input = b"{\"type\":\"app::launch\",\"path\":\"/Applications/Safari.app\"}\n";
        let mut writer = RecordingWriter::default();

        run_ndjson_loop(io::Cursor::new(input), &mut writer, &mut session).unwrap();

        assert_eq!(writer.flushes, 1);
        assert_eq!(writer.writes.len(), 1);
    }

    #[test]
    fn writer_errors_are_returned() {
        let mut session = test_session(Vec::new(), true);
        let input = b"{\"type\":\"app::launch\",\"path\":\"/Applications/Safari.app\"}\n";
        let mut writer = FailingWriter;

        let error = run_ndjson_loop(io::Cursor::new(input), &mut writer, &mut session).unwrap_err();

        assert_eq!(error.kind(), ErrorKind::BrokenPipe);
    }

    fn test_session(roots: Vec<PathBuf>, launch_succeeds: bool) -> CoreSession {
        CoreSession::with_applications(Applications::with_roots_and_launcher_for_test(
            roots,
            move |_| Ok(status(launch_succeeds)),
        ))
    }

    #[cfg(unix)]
    fn status(success: bool) -> ExitStatus {
        if success {
            ExitStatus::from_raw(0)
        } else {
            ExitStatus::from_raw(1 << 8)
        }
    }

    fn output_lines(output: &[u8]) -> Vec<&str> {
        std::str::from_utf8(output)
            .unwrap()
            .lines()
            .collect::<Vec<_>>()
    }

    fn json_type(line: &str) -> String {
        let value: Value = serde_json::from_str(line).unwrap();
        value["type"].as_str().unwrap().to_string()
    }

    #[derive(Default)]
    struct RecordingWriter {
        writes: Vec<Vec<u8>>,
        flushes: usize,
    }

    impl Write for RecordingWriter {
        fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
            self.writes.push(buf.to_vec());
            Ok(buf.len())
        }

        fn flush(&mut self) -> io::Result<()> {
            self.flushes += 1;
            Ok(())
        }
    }

    struct FailingWriter;

    impl Write for FailingWriter {
        fn write(&mut self, _buf: &[u8]) -> io::Result<usize> {
            Err(io::Error::new(ErrorKind::BrokenPipe, "writer failed"))
        }

        fn flush(&mut self) -> io::Result<()> {
            Ok(())
        }
    }

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("launchkick-transport-{nanos}"))
    }
}
