use plist::Value;
use std::{
    collections::HashSet,
    error::Error,
    fmt, fs, io,
    path::{Path, PathBuf},
    process::{Command, ExitStatus},
    sync::{Arc, Mutex},
    thread,
    time::{Duration, Instant},
};

const DISCOVERY_REFRESH_AFTER: Duration = Duration::from_secs(30);

pub struct Applications {
    roots: Vec<PathBuf>,
    launcher: Box<dyn ApplicationLauncher>,
    discovery_cache: Arc<Mutex<DiscoveryCache>>,
    refresh_after: Duration,
}

#[derive(Default)]
struct DiscoveryCache {
    applications: Option<Vec<Application>>,
    last_refresh: Option<Instant>,
    refresh_in_flight: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Application {
    pub name: String,
    pub path: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum LaunchError {
    EmptyPath,
    OpenFailed { path: String, message: String },
    OpenExitedUnsuccessfully { path: String, code: Option<i32> },
}

impl fmt::Display for LaunchError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LaunchError::EmptyPath => write!(f, "application path is empty"),
            LaunchError::OpenFailed { path, message } => {
                write!(f, "failed to launch {path}: {message}")
            }
            LaunchError::OpenExitedUnsuccessfully { path, code } => match code {
                Some(code) => write!(f, "failed to launch {path}: open exited with status {code}"),
                None => write!(f, "failed to launch {path}: open terminated by signal"),
            },
        }
    }
}

impl Error for LaunchError {}

impl Applications {
    pub fn system() -> Self {
        Self {
            roots: default_application_roots(),
            launcher: Box::new(OpenApplicationLauncher),
            discovery_cache: Arc::new(Mutex::new(DiscoveryCache::default())),
            refresh_after: DISCOVERY_REFRESH_AFTER,
        }
    }

    pub fn list(&self) -> Vec<Application> {
        let (cached_applications, should_refresh) = {
            let cache = self.discovery_cache.lock().unwrap();
            let cached_applications = cache.applications.clone();
            let should_refresh = cached_applications.is_some()
                && !cache.refresh_in_flight
                && cache
                    .last_refresh
                    .is_none_or(|last_refresh| last_refresh.elapsed() >= self.refresh_after);
            (cached_applications, should_refresh)
        };

        if let Some(cached_applications) = cached_applications {
            if should_refresh {
                self.refresh_in_background();
            }
            return cached_applications;
        }

        self.discover_and_store()
    }

    pub fn launch(&self, path: &str) -> Result<(), LaunchError> {
        if path.trim().is_empty() {
            return Err(LaunchError::EmptyPath);
        }

        let status = self
            .launcher
            .launch(path)
            .map_err(|error| LaunchError::OpenFailed {
                path: path.to_string(),
                message: error.to_string(),
            })?;

        if status.success() {
            Ok(())
        } else {
            Err(LaunchError::OpenExitedUnsuccessfully {
                path: path.to_string(),
                code: status.code(),
            })
        }
    }

    #[cfg(test)]
    pub(crate) fn with_roots_and_launcher_for_test<F>(roots: Vec<PathBuf>, launcher: F) -> Self
    where
        F: Fn(&str) -> io::Result<ExitStatus> + Send + Sync + 'static,
    {
        Self {
            roots,
            launcher: Box::new(ClosureApplicationLauncher { launch: launcher }),
            discovery_cache: Arc::new(Mutex::new(DiscoveryCache::default())),
            refresh_after: DISCOVERY_REFRESH_AFTER,
        }
    }

    #[cfg(test)]
    pub(crate) fn with_roots_launcher_and_refresh_for_test<F>(
        roots: Vec<PathBuf>,
        refresh_after: Duration,
        launcher: F,
    ) -> Self
    where
        F: Fn(&str) -> io::Result<ExitStatus> + Send + Sync + 'static,
    {
        Self {
            roots,
            launcher: Box::new(ClosureApplicationLauncher { launch: launcher }),
            discovery_cache: Arc::new(Mutex::new(DiscoveryCache::default())),
            refresh_after,
        }
    }

    fn discover_and_store(&self) -> Vec<Application> {
        let applications = discover_in_roots(&self.roots);
        let mut cache = self.discovery_cache.lock().unwrap();
        cache.applications = Some(applications.clone());
        cache.last_refresh = Some(Instant::now());
        cache.refresh_in_flight = false;
        applications
    }

    fn refresh_in_background(&self) {
        {
            let mut cache = self.discovery_cache.lock().unwrap();
            if cache.refresh_in_flight {
                return;
            }
            cache.refresh_in_flight = true;
        }

        let roots = self.roots.clone();
        let discovery_cache = Arc::clone(&self.discovery_cache);

        thread::spawn(move || {
            let applications = discover_in_roots(&roots);
            let mut cache = discovery_cache.lock().unwrap();
            cache.applications = Some(applications);
            cache.last_refresh = Some(Instant::now());
            cache.refresh_in_flight = false;
        });
    }
}

trait ApplicationLauncher: Send + Sync {
    fn launch(&self, path: &str) -> io::Result<ExitStatus>;
}

struct OpenApplicationLauncher;

impl ApplicationLauncher for OpenApplicationLauncher {
    fn launch(&self, path: &str) -> io::Result<ExitStatus> {
        Command::new("open").arg(path).status()
    }
}

#[cfg(test)]
struct ClosureApplicationLauncher<F> {
    launch: F,
}

#[cfg(test)]
impl<F> ApplicationLauncher for ClosureApplicationLauncher<F>
where
    F: Fn(&str) -> io::Result<ExitStatus> + Send + Sync,
{
    fn launch(&self, path: &str) -> io::Result<ExitStatus> {
        (self.launch)(path)
    }
}

fn discover_in_roots(roots: &[PathBuf]) -> Vec<Application> {
    let mut seen_paths = HashSet::new();
    let mut applications = Vec::new();

    for root in roots {
        discover_in_directory(root, &mut seen_paths, &mut applications);
    }

    applications.sort_by(|left, right| {
        left.name
            .to_lowercase()
            .cmp(&right.name.to_lowercase())
            .then_with(|| left.path.cmp(&right.path))
    });

    applications
}

fn default_application_roots() -> Vec<PathBuf> {
    let mut roots = vec![
        PathBuf::from("/Applications"),
        PathBuf::from("/System/Applications"),
        PathBuf::from("/System/Applications/Utilities"),
        PathBuf::from("/System/Library/CoreServices/Applications"),
    ];

    if let Some(home) = std::env::var_os("HOME") {
        roots.push(PathBuf::from(home).join("Applications"));
    }

    roots
}

fn discover_in_directory(
    directory: &Path,
    seen_paths: &mut HashSet<String>,
    applications: &mut Vec<Application>,
) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };

    for entry in entries.flatten() {
        let path = entry.path();
        let Ok(file_type) = entry.file_type() else {
            continue;
        };

        if !file_type.is_dir() {
            continue;
        }

        if is_application_bundle(&path) {
            if let Some(application) = application_from_bundle(&path)
                && seen_paths.insert(application.path.clone())
            {
                applications.push(application);
            }
            continue;
        }

        discover_in_directory(&path, seen_paths, applications);
    }
}

fn is_application_bundle(path: &Path) -> bool {
    path.extension()
        .and_then(|extension| extension.to_str())
        .is_some_and(|extension| extension.eq_ignore_ascii_case("app"))
}

fn application_from_bundle(path: &Path) -> Option<Application> {
    let plist = Value::from_file(path.join("Contents/Info.plist")).ok();

    if plist
        .as_ref()
        .and_then(|value| value.as_dictionary())
        .and_then(|dictionary| dictionary.get("LSUIElement"))
        .and_then(|value| value.as_boolean())
        .unwrap_or(false)
    {
        return None;
    }

    let name = plist
        .as_ref()
        .and_then(|value| value.as_dictionary())
        .and_then(|dictionary| {
            dictionary
                .get("CFBundleDisplayName")
                .or_else(|| dictionary.get("CFBundleName"))
        })
        .and_then(|value| value.as_string())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
        .or_else(|| fallback_name(path))?;

    let path = fs::canonicalize(path).unwrap_or_else(|_| path.to_path_buf());

    Some(Application {
        name,
        path: path.to_string_lossy().into_owned(),
    })
}

fn fallback_name(path: &Path) -> Option<String> {
    path.file_stem()
        .and_then(|name| name.to_str())
        .map(str::trim)
        .filter(|name| !name.is_empty())
        .map(ToOwned::to_owned)
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::{
        sync::atomic::{AtomicUsize, Ordering},
        thread,
        time::{SystemTime, UNIX_EPOCH},
    };

    static NEXT_TEMP_DIR_ID: AtomicUsize = AtomicUsize::new(0);

    #[cfg(unix)]
    use std::os::unix::process::ExitStatusExt;

    #[test]
    fn lists_user_launchable_applications_from_bundle_metadata() {
        let root = unique_temp_dir();
        write_info_plist(
            &root.join("Safari.app"),
            r#"
            <key>CFBundleDisplayName</key>
            <string>Safari Browser</string>
            "#,
        );
        write_info_plist(
            &root.join("Helpers/Menu Agent.app"),
            r#"
            <key>CFBundleName</key>
            <string>Menu Agent</string>
            <key>LSUIElement</key>
            <true/>
            "#,
        );

        let applications = test_applications(vec![root.clone()]).list();

        assert_eq!(applications.len(), 1);
        assert_eq!(applications[0].name, "Safari Browser");
        assert!(applications[0].path.ends_with("Safari.app"));

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn falls_back_to_bundle_name_then_file_stem() {
        let root = unique_temp_dir();
        write_info_plist(
            &root.join("Calendar.app"),
            r#"
            <key>CFBundleName</key>
            <string>Calendar Bundle</string>
            "#,
        );
        fs::create_dir_all(root.join("Notes.app/Contents")).unwrap();

        let applications = test_applications(vec![root.clone()]).list();
        let names = applications
            .iter()
            .map(|application| application.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(names, vec!["Calendar Bundle", "Notes"]);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn lists_nested_apps_and_sorts_case_insensitively() {
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("Utilities/zoom.app/Contents")).unwrap();
        fs::create_dir_all(root.join("Archive.app/Contents")).unwrap();

        let applications = test_applications(vec![root.clone()]).list();
        let names = applications
            .iter()
            .map(|application| application.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(names, vec!["Archive", "zoom"]);

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn ignores_missing_roots() {
        let root = unique_temp_dir();

        let applications = test_applications(vec![root]).list();

        assert!(applications.is_empty());
    }

    #[test]
    fn caches_discovered_applications_across_queries() {
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("Safari.app/Contents")).unwrap();
        let applications = test_applications(vec![root.clone()]);

        let first = applications.list();

        fs::create_dir_all(root.join("Notes.app/Contents")).unwrap();
        let second = applications.list();

        assert_eq!(first, second);
        assert_eq!(second.len(), 1);
        assert_eq!(second[0].name, "Safari");

        let _ = fs::remove_dir_all(root);
    }

    #[test]
    fn stale_while_revalidate_refreshes_in_background_for_later_queries() {
        let root = unique_temp_dir();
        fs::create_dir_all(root.join("Safari.app/Contents")).unwrap();
        let applications = Applications::with_roots_launcher_and_refresh_for_test(
            vec![root.clone()],
            Duration::ZERO,
            |_| Ok(success_status()),
        );

        let first = applications.list();
        fs::create_dir_all(root.join("Notes.app/Contents")).unwrap();

        let stale = applications.list();
        assert_eq!(stale, first);

        let refreshed = wait_for_applications(&applications, 2, Duration::from_secs(1));
        let names = refreshed
            .iter()
            .map(|application| application.name.as_str())
            .collect::<Vec<_>>();

        assert_eq!(names, vec!["Notes", "Safari"]);

        let _ = fs::remove_dir_all(root);
    }

    #[cfg(unix)]
    #[test]
    fn launch_returns_ok_when_open_succeeds() {
        let applications = Applications::with_roots_and_launcher_for_test(Vec::new(), |_| {
            Ok(ExitStatus::from_raw(0))
        });

        assert_eq!(applications.launch("/Applications/Safari.app"), Ok(()));
    }

    #[test]
    fn launch_rejects_empty_path() {
        let applications = test_applications(Vec::new());

        assert_eq!(applications.launch("  "), Err(LaunchError::EmptyPath));
    }

    #[cfg(unix)]
    #[test]
    fn launch_maps_spawn_errors() {
        let applications = Applications::with_roots_and_launcher_for_test(Vec::new(), |_| {
            Err(io::Error::new(io::ErrorKind::NotFound, "missing open"))
        });

        assert_eq!(
            applications.launch("/Applications/Safari.app"),
            Err(LaunchError::OpenFailed {
                path: "/Applications/Safari.app".to_string(),
                message: "missing open".to_string(),
            })
        );
    }

    #[cfg(unix)]
    #[test]
    fn launch_maps_unsuccessful_exit_status() {
        let applications = Applications::with_roots_and_launcher_for_test(Vec::new(), |_| {
            Ok(ExitStatus::from_raw(1 << 8))
        });

        assert_eq!(
            applications.launch("/Applications/Missing.app"),
            Err(LaunchError::OpenExitedUnsuccessfully {
                path: "/Applications/Missing.app".to_string(),
                code: Some(1),
            })
        );
    }

    fn test_applications(roots: Vec<PathBuf>) -> Applications {
        Applications::with_roots_and_launcher_for_test(roots, |_| Ok(success_status()))
    }

    fn wait_for_applications(
        applications: &Applications,
        expected_count: usize,
        timeout: Duration,
    ) -> Vec<Application> {
        let deadline = Instant::now() + timeout;

        loop {
            let snapshot = applications.list();
            if snapshot.len() == expected_count {
                return snapshot;
            }
            assert!(
                Instant::now() < deadline,
                "timed out waiting for refreshed applications"
            );
            thread::sleep(Duration::from_millis(10));
        }
    }

    #[cfg(unix)]
    fn success_status() -> ExitStatus {
        ExitStatus::from_raw(0)
    }

    fn write_info_plist(bundle_path: &Path, contents: &str) {
        let contents_path = bundle_path.join("Contents");
        fs::create_dir_all(&contents_path).unwrap();
        fs::write(
            contents_path.join("Info.plist"),
            format!(
                r#"<?xml version="1.0" encoding="UTF-8"?>
<!DOCTYPE plist PUBLIC "-//Apple//DTD PLIST 1.0//EN" "http://www.apple.com/DTDs/PropertyList-1.0.dtd">
<plist version="1.0">
<dict>
{contents}
</dict>
</plist>
"#
            ),
        )
        .unwrap();
    }

    fn unique_temp_dir() -> PathBuf {
        let nanos = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_nanos();
        let id = NEXT_TEMP_DIR_ID.fetch_add(1, Ordering::Relaxed);
        std::env::temp_dir().join(format!(
            "launchkick-apps-{}-{nanos}-{id}",
            std::process::id()
        ))
    }
}
