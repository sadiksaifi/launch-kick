use super::{
    action::{ActionBinding, ActionExecutionError, ActionExecutor},
    command_source::{CommandSource, CommandSourceError},
    session_state::LauncherResultRecord,
};
use crate::{
    applications::{Application, Applications},
    ipc::{IconDescriptor, LauncherAction, LauncherResult},
};
use std::sync::Arc;

const APPLICATION_SOURCE: &str = "applications";
const OPEN_ACTION: &str = "open";
const APPLICATION_RESULT_PREFIX: &str = "application:";

pub(crate) struct ApplicationCommandSource {
    applications: Arc<Applications>,
}

impl ApplicationCommandSource {
    pub(crate) fn system() -> Self {
        Self::with_applications(Applications::system())
    }

    pub(crate) fn with_applications(applications: Applications) -> Self {
        Self {
            applications: Arc::new(applications),
        }
    }
}

impl CommandSource for ApplicationCommandSource {
    fn results_for_query(
        &self,
        query: &str,
    ) -> Result<Vec<LauncherResultRecord>, CommandSourceError> {
        let normalized_query = query.trim().to_lowercase();
        let mut records = self
            .applications
            .list()
            .into_iter()
            .filter(|application| application_matches(application, &normalized_query))
            .map(|application| application_record(application, Arc::clone(&self.applications)))
            .collect::<Vec<_>>();

        records.sort_by(|left, right| {
            let left_result = left.as_result();
            let right_result = right.as_result();

            result_rank(&left_result.title, &normalized_query)
                .cmp(&result_rank(&right_result.title, &normalized_query))
                .then_with(|| {
                    left_result
                        .title
                        .to_lowercase()
                        .cmp(&right_result.title.to_lowercase())
                })
                .then_with(|| left_result.id.cmp(&right_result.id))
        });

        Ok(records)
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

fn application_record(
    application: Application,
    applications: Arc<Applications>,
) -> LauncherResultRecord {
    let path = application.path;
    let result = LauncherResult {
        id: format!("{APPLICATION_RESULT_PREFIX}{path}"),
        title: application.name,
        subtitle: Some(path.clone()),
        source: APPLICATION_SOURCE.to_string(),
        icon: Some(IconDescriptor {
            kind: "file".to_string(),
            value: path.clone(),
        }),
        actions: Vec::new(),
    };
    let binding = ActionBinding::new(
        LauncherAction {
            id: OPEN_ACTION.to_string(),
            title: "Open".to_string(),
        },
        OpenApplicationAction { applications, path },
    );

    LauncherResultRecord::new(result).with_action(binding)
}

struct OpenApplicationAction {
    applications: Arc<Applications>,
    path: String,
}

impl ActionExecutor for OpenApplicationAction {
    fn execute(&self) -> Result<(), ActionExecutionError> {
        self.applications
            .launch(&self.path)
            .map_err(|error| ActionExecutionError::new(error.to_string()))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::launcher::session_state::LauncherSessionState;
    use std::{
        fs,
        path::{Path, PathBuf},
        time::{SystemTime, UNIX_EPOCH},
    };

    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;

    #[test]
    fn produces_stable_application_result_records() {
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("Safari.app/Contents")).unwrap();
        let safari_path = canonical_string(&root.join("Safari.app"));
        let source = ApplicationCommandSource::with_applications(test_applications(
            vec![root.clone()],
            true,
        ));

        let records = source.results_for_query("").unwrap();
        let result = records[0].as_result();

        assert_eq!(result.id, format!("application:{safari_path}"));
        assert_eq!(result.title, "Safari");
        assert_eq!(result.subtitle.as_deref(), Some(safari_path.as_str()));
        assert_eq!(result.source, "applications");
        assert_eq!(
            result.icon.as_ref().map(|icon| icon.kind.as_str()),
            Some("file")
        );
        assert_eq!(
            result.icon.as_ref().map(|icon| icon.value.as_str()),
            Some(safari_path.as_str())
        );
        assert_eq!(result.actions.len(), 1);
        assert_eq!(result.actions[0].id, "open");
        assert_eq!(result.actions[0].title, "Open");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn filters_and_ranks_application_results() {
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("Notes.app/Contents")).unwrap();
        fs::create_dir_all(root.join("Safari.app/Contents")).unwrap();
        fs::create_dir_all(root.join("Safe Exam Browser.app/Contents")).unwrap();
        let source = ApplicationCommandSource::with_applications(test_applications(
            vec![root.clone()],
            true,
        ));

        let records = source.results_for_query("saf").unwrap();
        let titles = records
            .iter()
            .map(|record| record.as_result().title.as_str())
            .collect::<Vec<_>>();

        assert_eq!(titles, vec!["Safari", "Safe Exam Browser"]);

        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn application_open_action_executes_launch() {
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("Safari.app/Contents")).unwrap();
        let safari_path = canonical_string(&root.join("Safari.app"));
        let source = ApplicationCommandSource::with_applications(test_applications(
            vec![root.clone()],
            true,
        ));
        let mut state = LauncherSessionState::default();
        state.replace_results(String::new(), source.results_for_query("").unwrap());

        state
            .resolve_action(&format!("application:{safari_path}"), "open")
            .unwrap()
            .execute()
            .unwrap();

        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn application_open_action_returns_launch_failure() {
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("Missing.app/Contents")).unwrap();
        let missing_path = canonical_string(&root.join("Missing.app"));
        let source = ApplicationCommandSource::with_applications(test_applications(
            vec![root.clone()],
            false,
        ));
        let mut state = LauncherSessionState::default();
        state.replace_results(String::new(), source.results_for_query("").unwrap());

        let error = state
            .resolve_action(&format!("application:{missing_path}"), "open")
            .unwrap()
            .execute()
            .unwrap_err();

        assert_eq!(
            error.to_string(),
            format!("failed to launch {missing_path}: open exited with status 1")
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
        std::env::temp_dir().join(format!("launchkick-application-source-{nanos}"))
    }
}
