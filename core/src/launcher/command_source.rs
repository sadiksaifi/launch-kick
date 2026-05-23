use super::{application_source::ApplicationCommandSource, session_state::LauncherResultRecord};

pub(crate) trait CommandSource {
    fn results_for_query(&self, query: &str) -> Vec<LauncherResultRecord>;
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

    pub(crate) fn results_for_query(&self, query: &str) -> Vec<LauncherResultRecord> {
        self.sources
            .iter()
            .flat_map(|source| source.results_for_query(query))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::LauncherResult;

    #[test]
    fn combines_results_in_source_order() {
        let sources = CommandSources::new(vec![
            Box::new(StaticSource::new("first")),
            Box::new(StaticSource::new("second")),
        ]);

        let records = sources.results_for_query("query");
        let titles = records
            .iter()
            .map(|record| record.as_result().title.as_str())
            .collect::<Vec<_>>();

        assert_eq!(titles, vec!["first", "second"]);
    }

    struct StaticSource {
        title: String,
    }

    impl StaticSource {
        fn new(title: &str) -> Self {
            Self {
                title: title.to_string(),
            }
        }
    }

    impl CommandSource for StaticSource {
        fn results_for_query(&self, _query: &str) -> Vec<LauncherResultRecord> {
            vec![LauncherResultRecord::new(LauncherResult {
                id: format!("test:{}", self.title),
                title: self.title.clone(),
                subtitle: None,
                source: "test".to_string(),
                icon: None,
                actions: Vec::new(),
            })]
        }
    }
}
