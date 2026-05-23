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

    pub(crate) fn results_for_query(&self, query: &str) -> Vec<LauncherResultRecord> {
        let mut seen_result_ids = HashSet::new();
        let mut records = Vec::new();

        for source in &self.sources {
            let Ok(source_records) = source.results_for_query(query) else {
                continue;
            };

            for record in source_records {
                if seen_result_ids.insert(record.result_id().to_string()) {
                    records.push(record);
                }
            }
        }

        records
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::LauncherResult;

    #[test]
    fn combines_results_in_source_order() {
        let sources = CommandSources::new(vec![
            Box::new(StaticSource::new("first", vec!["one", "two"])),
            Box::new(StaticSource::new("second", vec!["three"])),
        ]);

        let records = sources.results_for_query("query");
        let titles = result_titles(&records);

        assert_eq!(titles, vec!["one", "two", "three"]);
    }

    #[test]
    fn first_duplicate_result_id_wins() {
        let sources = CommandSources::new(vec![
            Box::new(StaticSource::with_records(
                "first",
                vec![
                    ("command:shared", "first shared"),
                    ("command:only-first", "only first"),
                ],
            )),
            Box::new(StaticSource::with_records(
                "second",
                vec![
                    ("command:shared", "second shared"),
                    ("command:only-second", "only second"),
                ],
            )),
        ]);

        let records = sources.results_for_query("query");

        assert_eq!(
            result_titles(&records),
            vec!["first shared", "only first", "only second"]
        );
    }

    #[test]
    fn failing_source_does_not_prevent_later_sources() {
        let sources = CommandSources::new(vec![
            Box::new(StaticSource::new("first", vec!["before"])),
            Box::new(FailingSource),
            Box::new(StaticSource::new("second", vec!["after"])),
        ]);

        let records = sources.results_for_query("query");

        assert_eq!(result_titles(&records), vec!["before", "after"]);
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
    }

    impl StaticSource {
        fn new(source: &'static str, titles: Vec<&str>) -> Self {
            Self {
                source,
                records: titles
                    .into_iter()
                    .map(|title| (format!("{source}:{title}"), title.to_string()))
                    .collect(),
            }
        }

        fn with_records(source: &'static str, records: Vec<(&str, &str)>) -> Self {
            Self {
                source,
                records: records
                    .into_iter()
                    .map(|(id, title)| (id.to_string(), title.to_string()))
                    .collect(),
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
                })
                .collect())
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
