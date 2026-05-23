use crate::ipc::{self, ClientMessage, ServerMessage};
use std::io::{self, BufRead, Write};

pub fn run_ndjson_loop<R, W, H>(reader: R, mut writer: W, mut handle_message: H) -> io::Result<()>
where
    R: BufRead,
    W: Write,
    H: FnMut(ClientMessage) -> Vec<ServerMessage>,
{
    for line in reader.lines() {
        let line = line?;
        let Ok(message) = ipc::decode_client_line(&line) else {
            continue;
        };

        for response in handle_message(message) {
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
    use crate::ipc::{LauncherAction, LauncherResult};
    use serde_json::Value;
    use std::io::ErrorKind;

    #[test]
    fn eof_exits_cleanly() {
        let mut messages = Vec::new();
        let mut output = Vec::new();

        run_ndjson_loop(io::Cursor::new(Vec::<u8>::new()), &mut output, |message| {
            messages.push(message);
            Vec::new()
        })
        .unwrap();

        assert!(output.is_empty());
        assert!(messages.is_empty());
    }

    #[test]
    fn malformed_lines_are_ignored_and_loop_continues() {
        let mut handler = RecordingHandler::with_response(ServerMessage::Results {
            query: "saf".to_string(),
            results: vec![test_result()],
        });
        let input = b"not json\n{\"type\":\"launcher::query\",\"query\":\"saf\"}\n";
        let mut output = Vec::new();

        run_ndjson_loop(io::Cursor::new(input), &mut output, |message| {
            handler.handle(message)
        })
        .unwrap();

        let lines = output_lines(&output);
        assert_eq!(handler.messages.len(), 1);
        assert_eq!(lines.len(), 1);
        assert_eq!(json_type(lines[0]), "launcher::results");
    }

    #[test]
    fn multiple_messages_produce_multiple_server_lines() {
        let mut handler = RecordingHandler::with_responses(vec![
            ServerMessage::Results {
                query: String::new(),
                results: vec![test_result()],
            },
            ServerMessage::ActionResult {
                result_id: "application:/Applications/Safari.app".to_string(),
                action_id: "open".to_string(),
                ok: true,
                error: None,
            },
        ]);
        let input = b"{\"type\":\"launcher::query\",\"query\":\"\"}\n{\"type\":\"launcher::execute\",\"result_id\":\"application:/Applications/Safari.app\",\"action_id\":\"open\"}\n";
        let mut output = Vec::new();

        run_ndjson_loop(io::Cursor::new(input), &mut output, |message| {
            handler.handle(message)
        })
        .unwrap();

        let lines = output_lines(&output);
        assert_eq!(handler.messages.len(), 2);
        assert_eq!(lines.len(), 2);
        assert_eq!(json_type(lines[0]), "launcher::results");
        assert_eq!(json_type(lines[1]), "launcher::action::result");
    }

    #[test]
    fn flushes_after_each_server_message() {
        let mut handler = RecordingHandler::with_response(ServerMessage::ActionResult {
            result_id: "application:/Applications/Safari.app".to_string(),
            action_id: "open".to_string(),
            ok: true,
            error: None,
        });
        let input = b"{\"type\":\"launcher::execute\",\"result_id\":\"application:/Applications/Safari.app\",\"action_id\":\"open\"}\n";
        let mut writer = RecordingWriter::default();

        run_ndjson_loop(io::Cursor::new(input), &mut writer, |message| {
            handler.handle(message)
        })
        .unwrap();

        assert_eq!(writer.flushes, 1);
        assert_eq!(writer.writes.len(), 1);
    }

    #[test]
    fn writer_errors_are_returned() {
        let mut handler = RecordingHandler::with_response(ServerMessage::ActionResult {
            result_id: "application:/Applications/Safari.app".to_string(),
            action_id: "open".to_string(),
            ok: true,
            error: None,
        });
        let input = b"{\"type\":\"launcher::execute\",\"result_id\":\"application:/Applications/Safari.app\",\"action_id\":\"open\"}\n";
        let mut writer = FailingWriter;

        let error = run_ndjson_loop(io::Cursor::new(input), &mut writer, |message| {
            handler.handle(message)
        })
        .unwrap_err();

        assert_eq!(error.kind(), ErrorKind::BrokenPipe);
    }

    #[derive(Default)]
    struct RecordingHandler {
        messages: Vec<ClientMessage>,
        responses: Vec<ServerMessage>,
    }

    impl RecordingHandler {
        fn with_response(response: ServerMessage) -> Self {
            Self::with_responses(vec![response])
        }

        fn with_responses(responses: Vec<ServerMessage>) -> Self {
            Self {
                messages: Vec::new(),
                responses,
            }
        }

        fn handle(&mut self, message: ClientMessage) -> Vec<ServerMessage> {
            self.messages.push(message);
            if self.responses.is_empty() {
                Vec::new()
            } else {
                vec![self.responses.remove(0)]
            }
        }
    }

    fn test_result() -> LauncherResult {
        LauncherResult {
            id: "application:/Applications/Safari.app".to_string(),
            title: "Safari".to_string(),
            subtitle: Some("/Applications/Safari.app".to_string()),
            source: "applications".to_string(),
            icon: None,
            actions: vec![LauncherAction {
                id: "open".to_string(),
                title: "Open".to_string(),
            }],
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
}
