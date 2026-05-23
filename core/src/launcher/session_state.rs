use super::action::ActionBinding;
use crate::ipc::LauncherResult;
use std::collections::HashMap;

#[derive(Debug, Clone)]
pub(crate) struct LauncherResultRecord {
    result: LauncherResult,
    actions: HashMap<String, ActionBinding>,
}

impl LauncherResultRecord {
    pub(crate) fn new(result: LauncherResult) -> Self {
        Self {
            result,
            actions: HashMap::new(),
        }
    }

    pub(crate) fn with_action(mut self, binding: ActionBinding) -> Self {
        let action = binding.renderable_action().clone();
        self.actions.insert(binding.id().to_string(), binding);
        self.result.actions.push(action);
        self
    }

    pub(crate) fn as_result(&self) -> &LauncherResult {
        &self.result
    }

    pub(crate) fn result_id(&self) -> &str {
        &self.result.id
    }

    pub(crate) fn action(&self, action_id: &str) -> Option<&ActionBinding> {
        self.actions.get(action_id)
    }

    fn into_parts(self) -> (LauncherResult, HashMap<String, ActionBinding>) {
        (self.result, self.actions)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(super) enum ResolveActionError {
    UnknownResult,
    UnknownAction,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct VisibleLauncherResults {
    query: String,
    results: Vec<LauncherResult>,
}

impl VisibleLauncherResults {
    pub(crate) fn into_server_parts(self) -> (String, Vec<LauncherResult>) {
        (self.query, self.results)
    }
}

#[derive(Debug, Default)]
pub(crate) struct LauncherSessionState {
    current_query: String,
    visible_results: Vec<LauncherResult>,
    known_actions: HashMap<String, HashMap<String, ActionBinding>>,
}

impl LauncherSessionState {
    pub(crate) fn replace_results(
        &mut self,
        query: String,
        records: Vec<LauncherResultRecord>,
    ) -> VisibleLauncherResults {
        self.current_query = query;
        self.visible_results.clear();

        for record in records {
            let (result, actions) = record.into_parts();
            self.known_actions.insert(result.id.clone(), actions);
            self.visible_results.push(result);
        }

        VisibleLauncherResults {
            query: self.current_query.clone(),
            results: self.visible_results.clone(),
        }
    }

    pub(super) fn resolve_action(
        &self,
        result_id: &str,
        action_id: &str,
    ) -> Result<&ActionBinding, ResolveActionError> {
        let Some(actions) = self.known_actions.get(result_id) else {
            return Err(ResolveActionError::UnknownResult);
        };

        actions
            .get(action_id)
            .ok_or(ResolveActionError::UnknownAction)
    }

    #[cfg(test)]
    fn visible_result_count(&self) -> usize {
        self.visible_results.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ipc::{LauncherAction, LauncherResult},
        launcher::action::{ActionExecutionError, ActionExecutor},
    };
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn replace_results_records_visible_snapshot_and_known_actions() {
        let mut state = LauncherSessionState::default();
        let executions = Arc::new(AtomicUsize::new(0));
        let record = result_record("command:safari", Arc::clone(&executions));

        let visible = state.replace_results("saf".to_string(), vec![record]);
        let (query, results) = visible.into_server_parts();

        assert_eq!(query, "saf");
        assert_eq!(results.len(), 1);
        assert_eq!(results[0].actions[0].id, "open");
        state
            .resolve_action("command:safari", "open")
            .unwrap()
            .execute()
            .unwrap();
        assert_eq!(executions.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn known_results_survive_later_query_replacement() {
        let mut state = LauncherSessionState::default();
        let executions = Arc::new(AtomicUsize::new(0));
        state.replace_results(
            String::new(),
            vec![result_record("command:safari", Arc::clone(&executions))],
        );
        state.replace_results("missing".to_string(), Vec::new());

        assert_eq!(state.visible_result_count(), 0);
        state
            .resolve_action("command:safari", "open")
            .unwrap()
            .execute()
            .unwrap();
        assert_eq!(executions.load(Ordering::Relaxed), 1);
    }

    #[test]
    fn resolve_action_distinguishes_unknown_result_and_unknown_action() {
        let mut state = LauncherSessionState::default();
        state.replace_results(
            String::new(),
            vec![result_record(
                "command:safari",
                Arc::new(AtomicUsize::new(0)),
            )],
        );

        assert_eq!(
            state
                .resolve_action("command:safari", "rename")
                .unwrap_err(),
            ResolveActionError::UnknownAction
        );
        assert_eq!(
            state.resolve_action("command:missing", "open").unwrap_err(),
            ResolveActionError::UnknownResult
        );
    }

    #[test]
    fn result_record_exposes_action_metadata_from_binding() {
        let record = result_record("command:safari", Arc::new(AtomicUsize::new(0)));

        assert_eq!(record.result_id(), "command:safari");
        assert_eq!(record.as_result().actions.len(), 1);
        assert_eq!(record.as_result().actions[0].id, "open");
        assert_eq!(record.as_result().actions[0].title, "Open");
        assert!(record.action("open").is_some());
    }

    fn result_record(id: &str, executions: Arc<AtomicUsize>) -> LauncherResultRecord {
        let binding = ActionBinding::new(
            LauncherAction {
                id: "open".to_string(),
                title: "Open".to_string(),
            },
            CountingAction { executions },
        );

        LauncherResultRecord::new(LauncherResult {
            id: id.to_string(),
            title: "Safari".to_string(),
            subtitle: None,
            source: "test".to_string(),
            icon: None,
            actions: Vec::new(),
        })
        .with_action(binding)
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
