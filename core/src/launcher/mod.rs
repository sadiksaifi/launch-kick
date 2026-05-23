mod action;
mod application_source;
mod command_source;
mod session_state;

use crate::ipc::{ClientMessage, ServerMessage};
use command_source::CommandSources;
use session_state::{LauncherSessionState, ResolveActionError};

pub struct CoreSession {
    command_sources: CommandSources,
    state: LauncherSessionState,
}

impl CoreSession {
    pub fn new() -> Self {
        Self {
            command_sources: CommandSources::system(),
            state: LauncherSessionState::default(),
        }
    }

    #[cfg(test)]
    pub(crate) fn with_applications(applications: crate::applications::Applications) -> Self {
        Self::with_command_sources(CommandSources::new(vec![Box::new(
            application_source::ApplicationCommandSource::with_applications(applications),
        )]))
    }

    #[cfg(test)]
    pub(crate) fn with_command_sources(command_sources: CommandSources) -> Self {
        Self {
            command_sources,
            state: LauncherSessionState::default(),
        }
    }

    pub fn handle_client_message(&mut self, message: ClientMessage) -> Vec<ServerMessage> {
        match message {
            ClientMessage::Query { query } => vec![self.handle_query(query)],
            ClientMessage::Execute {
                result_id,
                action_id,
            } => vec![self.execute_action(result_id, action_id)],
        }
    }

    fn handle_query(&mut self, query: String) -> ServerMessage {
        let records = self.command_sources.results_for_query(&query);
        self.state.replace_results(query, records);

        ServerMessage::Results {
            query: self.state.current_query().to_string(),
            results: self.state.visible_results().to_vec(),
        }
    }

    fn execute_action(&self, result_id: String, action_id: String) -> ServerMessage {
        let response = |ok, error| ServerMessage::ActionResult {
            result_id: result_id.clone(),
            action_id: action_id.clone(),
            ok,
            error,
        };

        match self.state.resolve_action(&result_id, &action_id) {
            Ok(binding) => match binding.execute() {
                Ok(()) => response(true, None),
                Err(error) => response(false, Some(error.to_string())),
            },
            Err(ResolveActionError::UnknownResult) => {
                response(false, Some(format!("unknown result: {result_id}")))
            }
            Err(ResolveActionError::UnknownAction) => {
                response(false, Some(format!("unknown action: {action_id}")))
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::applications::Applications;
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;

    #[test]
    fn query_message_returns_application_results() {
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("Safari.app/Contents")).unwrap();
        let mut session =
            CoreSession::with_applications(test_applications(vec![root.clone()], true));

        let response = session.handle_client_message(ClientMessage::Query {
            query: String::new(),
        });

        let safari_path = canonical_string(&root.join("Safari.app"));
        assert_eq!(
            response,
            vec![ServerMessage::Results {
                query: String::new(),
                results: vec![crate::ipc::LauncherResult {
                    id: format!("application:{safari_path}"),
                    title: "Safari".to_string(),
                    subtitle: Some(safari_path.clone()),
                    source: "applications".to_string(),
                    icon: Some(crate::ipc::IconDescriptor {
                        kind: "file".to_string(),
                        value: safari_path,
                    }),
                    actions: vec![crate::ipc::LauncherAction {
                        id: "open".to_string(),
                        title: "Open".to_string(),
                    }],
                }]
            }]
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn query_message_filters_and_ranks_results() {
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("Notes.app/Contents")).unwrap();
        fs::create_dir_all(root.join("Safari.app/Contents")).unwrap();
        fs::create_dir_all(root.join("Safe Exam Browser.app/Contents")).unwrap();
        let mut session =
            CoreSession::with_applications(test_applications(vec![root.clone()], true));

        let response = session.handle_client_message(ClientMessage::Query {
            query: "saf".to_string(),
        });

        let [ServerMessage::Results { results, .. }] = response.as_slice() else {
            panic!("expected results response");
        };
        let titles = results
            .iter()
            .map(|result| result.title.as_str())
            .collect::<Vec<_>>();
        assert_eq!(titles, vec!["Safari", "Safe Exam Browser"]);

        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn execute_message_returns_success_result_for_current_application_result() {
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("Safari.app/Contents")).unwrap();
        let safari_path = canonical_string(&root.join("Safari.app"));
        let mut session =
            CoreSession::with_applications(test_applications(vec![root.clone()], true));
        session.handle_client_message(ClientMessage::Query {
            query: String::new(),
        });

        let response = session.handle_client_message(ClientMessage::Execute {
            result_id: format!("application:{safari_path}"),
            action_id: "open".to_string(),
        });

        assert_eq!(
            response,
            vec![ServerMessage::ActionResult {
                result_id: format!("application:{safari_path}"),
                action_id: "open".to_string(),
                ok: true,
                error: None,
            }]
        );

        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn execute_message_can_use_a_known_result_after_a_new_query_arrives() {
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("Safari.app/Contents")).unwrap();
        let safari_path = canonical_string(&root.join("Safari.app"));
        let mut session =
            CoreSession::with_applications(test_applications(vec![root.clone()], true));
        session.handle_client_message(ClientMessage::Query {
            query: String::new(),
        });
        session.handle_client_message(ClientMessage::Query {
            query: "does not match visible result yet".to_string(),
        });

        let response = session.handle_client_message(ClientMessage::Execute {
            result_id: format!("application:{safari_path}"),
            action_id: "open".to_string(),
        });

        assert_eq!(
            response,
            vec![ServerMessage::ActionResult {
                result_id: format!("application:{safari_path}"),
                action_id: "open".to_string(),
                ok: true,
                error: None,
            }]
        );

        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn execute_message_returns_failure_result() {
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("Missing.app/Contents")).unwrap();
        let missing_path = canonical_string(&root.join("Missing.app"));
        let mut session =
            CoreSession::with_applications(test_applications(vec![root.clone()], false));
        session.handle_client_message(ClientMessage::Query {
            query: String::new(),
        });

        let response = session.handle_client_message(ClientMessage::Execute {
            result_id: format!("application:{missing_path}"),
            action_id: "open".to_string(),
        });

        assert_eq!(
            response,
            vec![ServerMessage::ActionResult {
                result_id: format!("application:{missing_path}"),
                action_id: "open".to_string(),
                ok: false,
                error: Some(format!(
                    "failed to launch {missing_path}: open exited with status 1"
                )),
            }]
        );

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn execute_message_rejects_unknown_result() {
        let mut session = CoreSession::with_applications(test_applications(Vec::new(), true));

        let response = session.handle_client_message(ClientMessage::Execute {
            result_id: "application:/Applications/Safari.app".to_string(),
            action_id: "open".to_string(),
        });

        assert_eq!(
            response,
            vec![ServerMessage::ActionResult {
                result_id: "application:/Applications/Safari.app".to_string(),
                action_id: "open".to_string(),
                ok: false,
                error: Some("unknown result: application:/Applications/Safari.app".to_string()),
            }]
        );
    }

    #[test]
    fn execute_message_rejects_unknown_action_for_known_result() {
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("Safari.app/Contents")).unwrap();
        let safari_path = canonical_string(&root.join("Safari.app"));
        let mut session =
            CoreSession::with_applications(test_applications(vec![root.clone()], true));
        session.handle_client_message(ClientMessage::Query {
            query: String::new(),
        });

        let response = session.handle_client_message(ClientMessage::Execute {
            result_id: format!("application:{safari_path}"),
            action_id: "rename".to_string(),
        });

        assert_eq!(
            response,
            vec![ServerMessage::ActionResult {
                result_id: format!("application:{safari_path}"),
                action_id: "rename".to_string(),
                ok: false,
                error: Some("unknown action: rename".to_string()),
            }]
        );

        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    fn test_applications(roots: Vec<PathBuf>, launch_succeeds: bool) -> Applications {
        Applications::with_roots_and_launcher_for_test(roots, move |_| {
            let status = if launch_succeeds {
                ExitStatusExt::from_raw(0)
            } else {
                ExitStatusExt::from_raw(1 << 8)
            };
            Ok(status)
        })
    }

    #[cfg(not(unix))]
    fn test_applications(roots: Vec<PathBuf>, _launch_succeeds: bool) -> Applications {
        Applications::with_roots_and_launcher_for_test(roots, |_| unimplemented!())
    }

    fn canonical_string(path: &Path) -> String {
        fs::canonicalize(path)
            .unwrap()
            .to_string_lossy()
            .into_owned()
    }

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        std::env::temp_dir().join(format!("launchkick-session-{nanos}"))
    }
}
