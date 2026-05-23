use crate::{
    calculator,
    ipc::{ClientMessage, ServerMessage},
};

pub struct CoreSession;

impl CoreSession {
    pub fn new() -> Self {
        Self
    }

    pub fn handle_client_message(&mut self, message: ClientMessage) -> Option<ServerMessage> {
        match message {
            ClientMessage::Input { text } => Some(ServerMessage::Result {
                value: calculator::evaluate(&text),
            }),
            ClientMessage::AppList | ClientMessage::AppLaunch { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn input_message_returns_calculator_result() {
        let mut session = CoreSession::new();

        let response = session.handle_client_message(ClientMessage::Input {
            text: "1 + 2".to_string(),
        });

        assert_eq!(
            response,
            Some(ServerMessage::Result {
                value: "3".to_string()
            })
        );
    }

    #[test]
    fn invalid_calculator_input_preserves_empty_result_behavior() {
        let mut session = CoreSession::new();

        let response = session.handle_client_message(ClientMessage::Input {
            text: "1 + nope".to_string(),
        });

        assert_eq!(
            response,
            Some(ServerMessage::Result {
                value: String::new()
            })
        );
    }
}
