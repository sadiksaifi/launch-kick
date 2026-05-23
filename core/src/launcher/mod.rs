use crate::{
    applications::{Application, Applications},
    ipc::{ClientMessage, IconDescriptor, LauncherAction, LauncherResult, ServerMessage},
    transport::MessageHandler,
};
use std::collections::HashMap;

const APPLICATION_SOURCE: &str = "applications";
const OPEN_ACTION: &str = "open";
const APPLICATION_RESULT_PREFIX: &str = "application:";

pub struct CoreSession {
    applications: Applications,
    current_query: String,
    current_results: Vec<LauncherResult>,
    known_results: HashMap<String, LauncherResult>,
}

impl CoreSession {
    pub fn new() -> Self {
        Self::with_applications(Applications::system())
    }

    pub(crate) fn with_applications(applications: Applications) -> Self {
        Self {
            applications,
            current_query: String::new(),
            current_results: Vec::new(),
            known_results: HashMap::new(),
        }
    }

    pub fn handle_client_message(&mut self, message: ClientMessage) -> Vec<ServerMessage> {
        match message {
            ClientMessage::Query { query } => {
                self.current_query = query;
                self.current_results = self.results_for_query(&self.current_query);
                for result in &self.current_results {
                    self.known_results.insert(result.id.clone(), result.clone());
                }

                vec![ServerMessage::Results {
                    query: self.current_query.clone(),
                    results: self.current_results.clone(),
                }]
            }
            ClientMessage::Execute {
                result_id,
                action_id,
            } => vec![self.execute_action(result_id, action_id)],
        }
    }

    fn results_for_query(&self, query: &str) -> Vec<LauncherResult> {
        let normalized_query = query.trim().to_lowercase();
        let mut results = self
            .applications
            .list()
            .into_iter()
            .filter(|application| application_matches(application, &normalized_query))
            .map(application_result)
            .collect::<Vec<_>>();

        results.sort_by(|left, right| {
            result_rank(&left.title, &normalized_query)
                .cmp(&result_rank(&right.title, &normalized_query))
                .then_with(|| left.title.to_lowercase().cmp(&right.title.to_lowercase()))
                .then_with(|| left.id.cmp(&right.id))
        });

        results
    }

    fn execute_action(&self, result_id: String, action_id: String) -> ServerMessage {
        let response = |ok, error| ServerMessage::ActionResult {
            result_id: result_id.clone(),
            action_id: action_id.clone(),
            ok,
            error,
        };

        if action_id != OPEN_ACTION {
            return response(false, Some(format!("unknown action: {action_id}")));
        }

        let Some(result) = self.known_results.get(&result_id) else {
            return response(false, Some(format!("unknown result: {result_id}")));
        };

        if result.source != APPLICATION_SOURCE {
            return response(
                false,
                Some(format!("unsupported result source: {}", result.source)),
            );
        }

        let Some(path) = result.id.strip_prefix(APPLICATION_RESULT_PREFIX) else {
            return response(
                false,
                Some(format!("invalid application result: {result_id}")),
            );
        };

        match self.applications.launch(path) {
            Ok(()) => response(true, None),
            Err(error) => response(false, Some(error.to_string())),
        }
    }
}

impl MessageHandler for CoreSession {
    fn handle_client_message(&mut self, message: ClientMessage) -> Vec<ServerMessage> {
        CoreSession::handle_client_message(self, message)
    }
}

fn application_matches(application: &Application, normalized_query: &str) -> bool {
    normalized_query.is_empty()
        || application.name.to_lowercase().contains(normalized_query)
        || application.path.to_lowercase().contains(normalized_query)
}

fn result_rank(title: &str, normalized_query: &str) -> usize {
    if normalized_query.is_empty() {
        return 0;
    }

    let normalized_title = title.to_lowercase();
    if normalized_title == normalized_query {
        0
    } else if normalized_title.starts_with(normalized_query) {
        1
    } else if normalized_title.contains(normalized_query) {
        2
    } else {
        3
    }
}

fn application_result(application: Application) -> LauncherResult {
    LauncherResult {
        id: format!("{APPLICATION_RESULT_PREFIX}{}", application.path),
        title: application.name,
        subtitle: Some(application.path.clone()),
        source: APPLICATION_SOURCE.to_string(),
        icon: Some(IconDescriptor {
            kind: "file".to_string(),
            value: application.path,
        }),
        actions: vec![LauncherAction {
            id: OPEN_ACTION.to_string(),
            title: "Open".to_string(),
        }],
    }
}

#[cfg(test)]
mod tests {
    use super::*;
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
                results: vec![application_result(Application {
                    name: "Safari".to_string(),
                    path: safari_path,
                })]
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
