use crate::{
    applications::{ApplicationService, SystemApplicationService},
    calculator,
    ipc::{Application, ClientMessage, ServerMessage},
};

pub struct CoreSession {
    application_service: Box<dyn ApplicationService>,
}

impl CoreSession {
    pub fn new() -> Self {
        Self::with_application_service(Box::new(SystemApplicationService))
    }

    pub fn with_applications(applications: Vec<Application>) -> Self {
        Self::with_application_service(Box::new(StaticApplicationService { applications }))
    }

    pub fn with_application_service(application_service: Box<dyn ApplicationService>) -> Self {
        Self { application_service }
    }

    pub fn handle_client_message(&mut self, message: ClientMessage) -> Option<ServerMessage> {
        match message {
            ClientMessage::Input { text } => Some(ServerMessage::Result {
                value: calculator::evaluate(&text),
            }),
            ClientMessage::AppList => Some(ServerMessage::AppList {
                apps: self.application_service.list_applications(),
            }),
            ClientMessage::AppLaunch { path } => {
                let _ = self.application_service.launch_application(&path);
                None
            }
        }
    }
}

struct StaticApplicationService {
    applications: Vec<Application>,
}

impl ApplicationService for StaticApplicationService {
    fn list_applications(&mut self) -> Vec<Application> {
        self.applications.clone()
    }

    fn launch_application(&mut self, _path: &str) -> std::io::Result<()> {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{cell::RefCell, rc::Rc};

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
    fn app_launch_message_launches_selected_application() {
        let service = Rc::new(RefCell::new(RecordingApplicationService::default()));
        let mut session = CoreSession::with_application_service(Box::new(service.clone()));

        let response = session.handle_client_message(ClientMessage::AppLaunch {
            path: "/Applications/Safari.app".to_string(),
        });

        assert_eq!(response, None);
        assert_eq!(
            service.borrow().launched_paths,
            vec!["/Applications/Safari.app".to_string()]
        );
    }

    #[derive(Default)]
    struct RecordingApplicationService {
        launched_paths: Vec<String>,
    }

    impl ApplicationService for Rc<RefCell<RecordingApplicationService>> {
        fn list_applications(&mut self) -> Vec<Application> {
            Vec::new()
        }

        fn launch_application(&mut self, path: &str) -> std::io::Result<()> {
            self.borrow_mut().launched_paths.push(path.to_string());
            Ok(())
        }
    }
}
