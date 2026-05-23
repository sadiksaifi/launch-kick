use super::{
    action::{ActionBinding, ActionExecutionError, ActionExecutor},
    command_source::{CommandSource, CommandSourceError},
    session_state::LauncherResultRecord,
};
use crate::{
    applications::{Application, Applications},
    ipc::{IconDescriptor, LauncherAction, LauncherResult},
};
use nucleo_matcher::{
    Config, Matcher, Utf32Str,
    pattern::{AtomKind, CaseMatching, Normalization, Pattern},
};
use std::{cmp::Reverse, sync::Arc};

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
        let pattern = query_pattern(query);
        let mut name_matcher = Matcher::default();
        let mut path_matcher = Matcher::new(Config::DEFAULT.match_paths());
        let mut buf = Vec::new();
        let mut applications = self
            .applications
            .list()
            .into_iter()
            .map(|application| {
                let rank = application_rank(
                    &application,
                    &pattern,
                    &mut name_matcher,
                    &mut path_matcher,
                    &mut buf,
                );
                (rank, application)
            })
            .collect::<Vec<_>>();

        applications.sort_by(|(left_rank, left), (right_rank, right)| {
            left_rank
                .cmp(right_rank)
                .then_with(|| left.name.to_lowercase().cmp(&right.name.to_lowercase()))
                .then_with(|| left.path.cmp(&right.path))
        });

        Ok(applications
            .into_iter()
            .map(|(_, application)| application_record(application, Arc::clone(&self.applications)))
            .collect())
    }
}

fn query_pattern(query: &str) -> Pattern {
    Pattern::new(
        query.trim(),
        CaseMatching::Ignore,
        Normalization::Smart,
        AtomKind::Fuzzy,
    )
}

fn application_rank(
    application: &Application,
    pattern: &Pattern,
    name_matcher: &mut Matcher,
    path_matcher: &mut Matcher,
    buf: &mut Vec<char>,
) -> (usize, Reverse<u32>) {
    if pattern.atoms.is_empty() {
        return (0, Reverse(0));
    }

    if let Some(score) = pattern.score(Utf32Str::new(&application.name, buf), name_matcher) {
        return (0, Reverse(score));
    }

    if let Some(score) = pattern.score(Utf32Str::new(&application.path, buf), path_matcher) {
        return (1, Reverse(score));
    }

    (2, Reverse(0))
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
    fn fuzzy_orders_application_results_without_filtering() {
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

        assert_eq!(titles, vec!["Safari", "Safe Exam Browser", "Notes"]);

        let fuzzy_records = source.results_for_query("nts").unwrap();
        let fuzzy_titles = fuzzy_records
            .iter()
            .map(|record| record.as_result().title.as_str())
            .collect::<Vec<_>>();

        assert_eq!(fuzzy_titles[0], "Notes");

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
        let records = source.results_for_query("").unwrap();
        let record = records
            .iter()
            .find(|record| record.result_id() == format!("application:{safari_path}"))
            .unwrap();

        record.action("open").unwrap().execute().unwrap();

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
        let records = source.results_for_query("").unwrap();
        let record = records
            .iter()
            .find(|record| record.result_id() == format!("application:{missing_path}"))
            .unwrap();

        let error = record.action("open").unwrap().execute().unwrap_err();

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
