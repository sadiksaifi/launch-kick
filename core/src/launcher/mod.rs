use crate::{
    applications::Applications,
    ipc::{ClientMessage, ServerMessage},
};

pub struct CoreSession {
    applications: Applications,
}

impl CoreSession {
    pub fn new() -> Self {
        Self::with_applications(Applications::system())
    }

    pub(crate) fn with_applications(applications: Applications) -> Self {
        Self { applications }
    }

    pub fn handle_client_message(&mut self, message: ClientMessage) -> Vec<ServerMessage> {
        match message {
            ClientMessage::AppList => vec![ServerMessage::AppList {
                apps: self.applications.list(),
            }],
            ClientMessage::AppLaunch { path } => {
                let response = match self.applications.launch(&path) {
                    Ok(()) => ServerMessage::AppLaunchResult {
                        path,
                        ok: true,
                        error: None,
                    },
                    Err(error) => ServerMessage::AppLaunchResult {
                        path,
                        ok: false,
                        error: Some(error.to_string()),
                    },
                };

                vec![response]
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::ipc::Application;
    use std::{
        fs,
        path::Path,
        path::PathBuf,
        time::{SystemTime, UNIX_EPOCH},
    };

    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;

    #[test]
    fn app_list_message_returns_available_applications() {
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("Safari.app/Contents")).unwrap();
        let mut session =
            CoreSession::with_applications(test_applications(vec![root.clone()], true));

        let response = session.handle_client_message(ClientMessage::AppList);

        assert_eq!(
            response,
            vec![ServerMessage::AppList {
                apps: vec![Application {
                    name: "Safari".to_string(),
                    path: canonical_string(&root.join("Safari.app")),
                }]
            }]
        );

        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn app_launch_message_returns_success_result() {
        let mut session = CoreSession::with_applications(test_applications(Vec::new(), true));

        let response = session.handle_client_message(ClientMessage::AppLaunch {
            path: "/Applications/Safari.app".to_string(),
        });

        assert_eq!(
            response,
            vec![ServerMessage::AppLaunchResult {
                path: "/Applications/Safari.app".to_string(),
                ok: true,
                error: None,
            }]
        );
    }

    #[cfg(unix)]
    #[test]
    fn app_launch_message_returns_failure_result() {
        let mut session = CoreSession::with_applications(test_applications(Vec::new(), false));

        let response = session.handle_client_message(ClientMessage::AppLaunch {
            path: "/Applications/Missing.app".to_string(),
        });

        assert_eq!(
            response,
            vec![ServerMessage::AppLaunchResult {
                path: "/Applications/Missing.app".to_string(),
                ok: false,
                error: Some(
                    "failed to launch /Applications/Missing.app: open exited with status 1"
                        .to_string()
                ),
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
