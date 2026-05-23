use crate::ipc::LauncherResult;
use std::collections::HashMap;

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ActionBinding {
    OpenApplication { path: String },
}

#[derive(Debug, Clone, PartialEq, Eq)]
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

    pub(crate) fn with_action(
        mut self,
        action_id: impl Into<String>,
        binding: ActionBinding,
    ) -> Self {
        self.actions.insert(action_id.into(), binding);
        self
    }

    pub(crate) fn as_result(&self) -> &LauncherResult {
        &self.result
    }

    fn into_parts(self) -> (LauncherResult, HashMap<String, ActionBinding>) {
        (self.result, self.actions)
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum ResolveActionError {
    UnknownResult,
    UnknownAction,
}

#[derive(Debug, Default)]
pub(crate) struct LauncherSessionState {
    current_query: String,
    visible_results: Vec<LauncherResult>,
    known_actions: HashMap<String, HashMap<String, ActionBinding>>,
}

impl LauncherSessionState {
    pub(crate) fn replace_results(&mut self, query: String, records: Vec<LauncherResultRecord>) {
        self.current_query = query;
        self.visible_results.clear();

        for record in records {
            let (result, actions) = record.into_parts();
            self.known_actions.insert(result.id.clone(), actions);
            self.visible_results.push(result);
        }
    }

    pub(crate) fn current_query(&self) -> &str {
        &self.current_query
    }

    pub(crate) fn visible_results(&self) -> &[LauncherResult] {
        &self.visible_results
    }

    pub(crate) fn resolve_action(
        &self,
        result_id: &str,
        action_id: &str,
    ) -> Result<ActionBinding, ResolveActionError> {
        let Some(actions) = self.known_actions.get(result_id) else {
            return Err(ResolveActionError::UnknownResult);
        };

        actions
            .get(action_id)
            .cloned()
            .ok_or(ResolveActionError::UnknownAction)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::{LauncherAction, LauncherResult};

    #[test]
    fn replace_results_tracks_visible_results_and_known_actions() {
        let mut state = LauncherSessionState::default();
        let record = result_record("application:/Applications/Safari.app");

        state.replace_results("saf".to_string(), vec![record]);

        assert_eq!(state.current_query(), "saf");
        assert_eq!(state.visible_results().len(), 1);
        assert_eq!(
            state.resolve_action("application:/Applications/Safari.app", "open"),
            Ok(ActionBinding::OpenApplication {
                path: "/Applications/Safari.app".to_string()
            })
        );
    }

    #[test]
    fn known_results_survive_later_query_replacement() {
        let mut state = LauncherSessionState::default();
        state.replace_results(
            String::new(),
            vec![result_record("application:/Applications/Safari.app")],
        );
        state.replace_results("missing".to_string(), Vec::new());

        assert!(state.visible_results().is_empty());
        assert_eq!(
            state.resolve_action("application:/Applications/Safari.app", "open"),
            Ok(ActionBinding::OpenApplication {
                path: "/Applications/Safari.app".to_string()
            })
        );
    }

    #[test]
    fn resolve_action_distinguishes_unknown_result_and_unknown_action() {
        let mut state = LauncherSessionState::default();
        state.replace_results(
            String::new(),
            vec![result_record("application:/Applications/Safari.app")],
        );

        assert_eq!(
            state.resolve_action("application:/Applications/Safari.app", "rename"),
            Err(ResolveActionError::UnknownAction)
        );
        assert_eq!(
            state.resolve_action("application:/Applications/Missing.app", "open"),
            Err(ResolveActionError::UnknownResult)
        );
    }

    fn result_record(id: &str) -> LauncherResultRecord {
        LauncherResultRecord::new(LauncherResult {
            id: id.to_string(),
            title: "Safari".to_string(),
            subtitle: Some("/Applications/Safari.app".to_string()),
            source: "applications".to_string(),
            icon: None,
            actions: vec![LauncherAction {
                id: "open".to_string(),
                title: "Open".to_string(),
            }],
        })
        .with_action(
            "open",
            ActionBinding::OpenApplication {
                path: "/Applications/Safari.app".to_string(),
            },
        )
    }
}
