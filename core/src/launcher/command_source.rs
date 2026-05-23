use super::{application_source::ApplicationCommandSource, session_state::LauncherResultRecord};
use std::{collections::HashSet, error::Error, fmt};

pub(crate) trait CommandSource {
    fn results_for_query(
        &self,
        query: &str,
    ) -> Result<Vec<LauncherResultRecord>, CommandSourceError>;
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct CommandSourceError {
    message: String,
}

impl CommandSourceError {
    #[allow(dead_code)]
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for CommandSourceError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for CommandSourceError {}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct SuppressedSourceFailure {
    source_index: usize,
    message: String,
}

impl SuppressedSourceFailure {
    #[cfg(test)]
    pub(crate) fn source_index(&self) -> usize {
        self.source_index
    }

    #[cfg(test)]
    pub(crate) fn message(&self) -> &str {
        &self.message
    }
}

#[derive(Debug, Default)]
pub(crate) struct CommandSourceQueryResult {
    records: Vec<LauncherResultRecord>,
    suppressed_failures: Vec<SuppressedSourceFailure>,
}

impl CommandSourceQueryResult {
    pub(crate) fn into_records(self) -> Vec<LauncherResultRecord> {
        self.records
    }

    #[cfg(test)]
    pub(crate) fn records(&self) -> &[LauncherResultRecord] {
        &self.records
    }

    #[cfg(test)]
    pub(crate) fn suppressed_failures(&self) -> &[SuppressedSourceFailure] {
        &self.suppressed_failures
    }
}

pub(crate) struct CommandSources {
    sources: Vec<Box<dyn CommandSource>>,
}

impl CommandSources {
    pub(crate) fn system() -> Self {
        Self::new(vec![Box::new(ApplicationCommandSource::system())])
    }

    pub(crate) fn new(sources: Vec<Box<dyn CommandSource>>) -> Self {
        Self { sources }
    }

    pub(crate) fn results_for_query(&self, query: &str) -> CommandSourceQueryResult {
        let mut seen_result_ids = HashSet::new();
        let mut outcome = CommandSourceQueryResult::default();

        for (source_index, source) in self.sources.iter().enumerate() {
            let source_records = match source.results_for_query(query) {
                Ok(records) => records,
                Err(error) => {
                    outcome.suppressed_failures.push(SuppressedSourceFailure {
                        source_index,
                        message: error.to_string(),
                    });
                    continue;
                }
            };

            for record in source_records {
                if seen_result_ids.insert(record.result_id().to_string()) {
                    outcome.records.push(record);
                }
            }
        }

        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        ipc::{LauncherAction, LauncherResult},
        launcher::action::{ActionBinding, ActionExecutionError, ActionExecutor},
    };
    use std::sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    };

    #[test]
    fn combines_results_in_source_order() {
        let sources = CommandSources::new(vec![
            Box::new(StaticSource::new("first", vec!["one", "two"])),
            Box::new(StaticSource::new("second", vec!["three"])),
        ]);

        let outcome = sources.results_for_query("query");
        let titles = result_titles(outcome.records());

        assert_eq!(titles, vec!["one", "two", "three"]);
    }

    #[test]
    fn first_duplicate_result_id_wins_with_its_action_binding() {
        let first_executions = Arc::new(AtomicUsize::new(0));
        let second_executions = Arc::new(AtomicUsize::new(0));
        let sources = CommandSources::new(vec![
            Box::new(StaticSource::with_records(
                "first",
                vec![("command:shared", "first shared")],
                Arc::clone(&first_executions),
            )),
            Box::new(StaticSource::with_records(
                "second",
                vec![
                    ("command:shared", "second shared"),
                    ("command:only-second", "only second"),
                ],
                Arc::clone(&second_executions),
            )),
        ]);

        let outcome = sources.results_for_query("query");
        let records = outcome.records();

        assert_eq!(result_titles(records), vec!["first shared", "only second"]);
        records[0].action("open").unwrap().execute().unwrap();
        assert_eq!(first_executions.load(Ordering::Relaxed), 1);
        assert_eq!(second_executions.load(Ordering::Relaxed), 0);
    }

    #[test]
    fn failing_source_records_failure_without_preventing_later_sources() {
        let sources = CommandSources::new(vec![
            Box::new(StaticSource::new("first", vec!["before"])),
            Box::new(FailingSource),
            Box::new(StaticSource::new("second", vec!["after"])),
        ]);

        let outcome = sources.results_for_query("query");

        assert_eq!(result_titles(outcome.records()), vec!["before", "after"]);
        assert_eq!(outcome.suppressed_failures().len(), 1);
        assert_eq!(outcome.suppressed_failures()[0].source_index(), 1);
        assert_eq!(outcome.suppressed_failures()[0].message(), "source failed");
    }

    fn result_titles(records: &[LauncherResultRecord]) -> Vec<&str> {
        records
            .iter()
            .map(|record| record.as_result().title.as_str())
            .collect()
    }

    struct StaticSource {
        source: &'static str,
        records: Vec<(String, String)>,
        executions: Arc<AtomicUsize>,
    }

    impl StaticSource {
        fn new(source: &'static str, titles: Vec<&str>) -> Self {
            Self {
                source,
                records: titles
                    .into_iter()
                    .map(|title| (format!("{source}:{title}"), title.to_string()))
                    .collect(),
                executions: Arc::new(AtomicUsize::new(0)),
            }
        }

        fn with_records(
            source: &'static str,
            records: Vec<(&str, &str)>,
            executions: Arc<AtomicUsize>,
        ) -> Self {
            Self {
                source,
                records: records
                    .into_iter()
                    .map(|(id, title)| (id.to_string(), title.to_string()))
                    .collect(),
                executions,
            }
        }
    }

    impl CommandSource for StaticSource {
        fn results_for_query(
            &self,
            _query: &str,
        ) -> Result<Vec<LauncherResultRecord>, CommandSourceError> {
            Ok(self
                .records
                .iter()
                .map(|(id, title)| {
                    LauncherResultRecord::new(LauncherResult {
                        id: id.clone(),
                        title: title.clone(),
                        subtitle: None,
                        source: self.source.to_string(),
                        icon: None,
                        actions: Vec::new(),
                    })
                    .with_action(ActionBinding::new(
                        LauncherAction {
                            id: "open".to_string(),
                            title: "Open".to_string(),
                        },
                        CountingAction {
                            executions: Arc::clone(&self.executions),
                        },
                    ))
                })
                .collect())
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

    struct FailingSource;

    impl CommandSource for FailingSource {
        fn results_for_query(
            &self,
            _query: &str,
        ) -> Result<Vec<LauncherResultRecord>, CommandSourceError> {
            Err(CommandSourceError::new("source failed"))
        }
    }
}
