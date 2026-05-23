use std::{error::Error, fmt, sync::Arc};

pub(crate) trait ActionExecutor: Send + Sync {
    fn execute(&self) -> Result<(), ActionExecutionError>;
}

#[derive(Clone)]
pub(crate) struct ActionBinding {
    executor: Arc<dyn ActionExecutor>,
}

impl ActionBinding {
    pub(crate) fn new(executor: impl ActionExecutor + 'static) -> Self {
        Self {
            executor: Arc::new(executor),
        }
    }

    pub(crate) fn execute(&self) -> Result<(), ActionExecutionError> {
        self.executor.execute()
    }
}

impl fmt::Debug for ActionBinding {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("ActionBinding").finish_non_exhaustive()
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) struct ActionExecutionError {
    message: String,
}

impl ActionExecutionError {
    pub(crate) fn new(message: impl Into<String>) -> Self {
        Self {
            message: message.into(),
        }
    }
}

impl fmt::Display for ActionExecutionError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str(&self.message)
    }
}

impl Error for ActionExecutionError {}
