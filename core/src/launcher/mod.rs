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
        let records = self
            .command_sources
            .results_for_query(&query)
            .into_records();
        let (query, results) = self
            .state
            .replace_results(query, records)
            .into_server_parts();

        ServerMessage::Results { query, results }
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
    use crate::{
        ipc::{LauncherAction, LauncherResult},
        launcher::{
            action::{ActionBinding, ActionExecutionError, ActionExecutor},
            command_source::{CommandSource, CommandSourceError},
            session_state::LauncherResultRecord,
        },
    };
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn query_message_returns_command_source_results() {
        let mut session = CoreSession::with_command_sources(CommandSources::new(vec![Box::new(
            FakeCommandSource::new(vec![record("command:safari", "Safari", Ok(()))]),
        )]));

        let response = session.handle_client_message(ClientMessage::Query {
            query: String::new(),
        });

        assert_eq!(
            response,
            vec![ServerMessage::Results {
                query: String::new(),
                results: vec![LauncherResult {
                    id: "command:safari".to_string(),
                    title: "Safari".to_string(),
                    subtitle: None,
                    source: "test".to_string(),
                    icon: None,
                    actions: vec![LauncherAction {
                        id: "open".to_string(),
                        title: "Open".to_string(),
                    }],
                }]
            }]
        );
    }

    #[test]
    fn query_message_uses_registry_policy() {
        let mut session = CoreSession::with_command_sources(CommandSources::new(vec![
            Box::new(FakeCommandSource::new(vec![record(
                "command:one",
                "One",
                Ok(()),
            )])),
            Box::new(FakeCommandSource::new(vec![record(
                "command:two",
                "Two",
                Ok(()),
            )])),
        ]));

        let response = session.handle_client_message(ClientMessage::Query {
            query: "anything".to_string(),
        });

        let [ServerMessage::Results { results, .. }] = response.as_slice() else {
            panic!("expected results response");
        };
        let titles = results
            .iter()
            .map(|result| result.title.as_str())
            .collect::<Vec<_>>();
        assert_eq!(titles, vec!["One", "Two"]);
    }

    #[test]
    fn execute_message_returns_success_result_for_current_result() {
        let executions = Arc::new(AtomicUsize::new(0));
        let mut session = CoreSession::with_command_sources(CommandSources::new(vec![Box::new(
            FakeCommandSource::new(vec![record_with_counter(
                "command:safari",
                "Safari",
                Arc::clone(&executions),
            )]),
        )]));
        session.handle_client_message(ClientMessage::Query {
            query: String::new(),
        });

        let response = session.handle_client_message(ClientMessage::Execute {
            result_id: "command:safari".to_string(),
            action_id: "open".to_string(),
        });

        assert_eq!(
            response,
            vec![ServerMessage::ActionResult {
                result_id: "command:safari".to_string(),
                action_id: "open".to_string(),
                ok: true,
                error: None,
            }]
        );
        assert_eq!(executions.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn execute_message_can_use_a_known_result_after_a_new_query_arrives() {
        let executions = Arc::new(AtomicUsize::new(0));
        let mut session = CoreSession::with_command_sources(CommandSources::new(vec![Box::new(
            QueryAwareCommandSource::new(
                "saf",
                vec![record_with_counter(
                    "command:safari",
                    "Safari",
                    Arc::clone(&executions),
                )],
            ),
        )]));
        session.handle_client_message(ClientMessage::Query {
            query: "saf".to_string(),
        });
        session.handle_client_message(ClientMessage::Query {
            query: "missing".to_string(),
        });

        let response = session.handle_client_message(ClientMessage::Execute {
            result_id: "command:safari".to_string(),
            action_id: "open".to_string(),
        });

        assert_eq!(
            response,
            vec![ServerMessage::ActionResult {
                result_id: "command:safari".to_string(),
                action_id: "open".to_string(),
                ok: true,
                error: None,
            }]
        );
        assert_eq!(executions.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn execute_message_returns_failure_result() {
        let mut session = CoreSession::with_command_sources(CommandSources::new(vec![Box::new(
            FakeCommandSource::new(vec![record("command:broken", "Broken", Err("boom"))]),
        )]));
        session.handle_client_message(ClientMessage::Query {
            query: String::new(),
        });

        let response = session.handle_client_message(ClientMessage::Execute {
            result_id: "command:broken".to_string(),
            action_id: "open".to_string(),
        });

        assert_eq!(
            response,
            vec![ServerMessage::ActionResult {
                result_id: "command:broken".to_string(),
                action_id: "open".to_string(),
                ok: false,
                error: Some("boom".to_string()),
            }]
        );
    }

    #[test]
    fn execute_message_rejects_unknown_result() {
        let mut session = CoreSession::with_command_sources(CommandSources::new(Vec::new()));

        let response = session.handle_client_message(ClientMessage::Execute {
            result_id: "command:missing".to_string(),
            action_id: "open".to_string(),
        });

        assert_eq!(
            response,
            vec![ServerMessage::ActionResult {
                result_id: "command:missing".to_string(),
                action_id: "open".to_string(),
                ok: false,
                error: Some("unknown result: command:missing".to_string()),
            }]
        );
    }

    #[test]
    fn execute_message_rejects_unknown_action_for_known_result() {
        let mut session = CoreSession::with_command_sources(CommandSources::new(vec![Box::new(
            FakeCommandSource::new(vec![record("command:safari", "Safari", Ok(()))]),
        )]));
        session.handle_client_message(ClientMessage::Query {
            query: String::new(),
        });

        let response = session.handle_client_message(ClientMessage::Execute {
            result_id: "command:safari".to_string(),
            action_id: "rename".to_string(),
        });

        assert_eq!(
            response,
            vec![ServerMessage::ActionResult {
                result_id: "command:safari".to_string(),
                action_id: "rename".to_string(),
                ok: false,
                error: Some("unknown action: rename".to_string()),
            }]
        );
    }

    fn record(id: &str, title: &str, outcome: Result<(), &str>) -> LauncherResultRecord {
        LauncherResultRecord::new(LauncherResult {
            id: id.to_string(),
            title: title.to_string(),
            subtitle: None,
            source: "test".to_string(),
            icon: None,
            actions: Vec::new(),
        })
        .with_action(ActionBinding::new(
            LauncherAction {
                id: "open".to_string(),
                title: "Open".to_string(),
            },
            FakeAction::new(outcome.map_err(ToOwned::to_owned)),
        ))
    }

    fn record_with_counter(
        id: &str,
        title: &str,
        executions: Arc<AtomicUsize>,
    ) -> LauncherResultRecord {
        LauncherResultRecord::new(LauncherResult {
            id: id.to_string(),
            title: title.to_string(),
            subtitle: None,
            source: "test".to_string(),
            icon: None,
            actions: Vec::new(),
        })
        .with_action(ActionBinding::new(
            LauncherAction {
                id: "open".to_string(),
                title: "Open".to_string(),
            },
            CountingAction { executions },
        ))
    }

    struct FakeCommandSource {
        records: Vec<LauncherResultRecord>,
    }

    impl FakeCommandSource {
        fn new(records: Vec<LauncherResultRecord>) -> Self {
            Self { records }
        }
    }

    impl CommandSource for FakeCommandSource {
        fn results_for_query(
            &self,
            _query: &str,
        ) -> Result<Vec<LauncherResultRecord>, CommandSourceError> {
            Ok(self.records.clone())
        }
    }

    struct QueryAwareCommandSource {
        matching_query: String,
        records: Vec<LauncherResultRecord>,
    }

    impl QueryAwareCommandSource {
        fn new(matching_query: &str, records: Vec<LauncherResultRecord>) -> Self {
            Self {
                matching_query: matching_query.to_string(),
                records,
            }
        }
    }

    impl CommandSource for QueryAwareCommandSource {
        fn results_for_query(
            &self,
            query: &str,
        ) -> Result<Vec<LauncherResultRecord>, CommandSourceError> {
            if query == self.matching_query {
                Ok(self.records.clone())
            } else {
                Ok(Vec::new())
            }
        }
    }

    struct FakeAction {
        outcome: Result<(), String>,
    }

    impl FakeAction {
        fn new(outcome: Result<(), String>) -> Self {
            Self { outcome }
        }
    }

    impl ActionExecutor for FakeAction {
        fn execute(&self) -> Result<(), ActionExecutionError> {
            self.outcome.clone().map_err(ActionExecutionError::new)
        }
    }

    struct CountingAction {
        executions: Arc<AtomicUsize>,
    }

    impl ActionExecutor for CountingAction {
        fn execute(&self) -> Result<(), ActionExecutionError> {
            self.executions.fetch_add(1, Ordering::Relaxed);
            Ok(())
        }
    }
}
