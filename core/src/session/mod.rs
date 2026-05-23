use crate::{
    calculator,
    ipc::{Application, ClientMessage, ServerMessage},
};

pub struct CoreSession {
    applications: Vec<Application>,
}

impl CoreSession {
    pub fn new() -> Self {
        Self::with_applications(Vec::new())
    }

    pub fn with_applications(applications: Vec<Application>) -> Self {
        Self { applications }
    }

    pub fn handle_client_message(&mut self, message: ClientMessage) -> Option<ServerMessage> {
        match message {
            ClientMessage::Input { text } => Some(ServerMessage::Result {
                value: calculator::evaluate(&text),
            }),
            ClientMessage::AppList => Some(ServerMessage::AppList {
                apps: self.applications.clone(),
            }),
            ClientMessage::AppLaunch { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn app_list_message_returns_available_applications() {
        let mut session = CoreSession::with_applications(vec![Application {
            name: "Safari".to_string(),
            path: "/Applications/Safari.app".to_string(),
        }]);

        let response = session.handle_client_message(ClientMessage::AppList);

        assert_eq!(
            response,
            Some(ServerMessage::AppList {
                apps: vec![Application {
                    name: "Safari".to_string(),
                    path: "/Applications/Safari.app".to_string(),
                }]
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
