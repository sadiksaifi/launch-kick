use crate::ipc::Application;
use plist::Value;
use std::{
    collections::HashSet,
    fs, io,
    path::{Path, PathBuf},
    process::Command,
};

pub trait ApplicationService {
    fn list_applications(&mut self) -> Vec<Application>;
    fn launch_application(&mut self, path: &str) -> io::Result<()>;
}

pub struct SystemApplicationService;

impl ApplicationService for SystemApplicationService {
    fn list_applications(&mut self) -> Vec<Application> {
        discover()
    }

    fn launch_application(&mut self, path: &str) -> io::Result<()> {
        Command::new("open").arg(path).status()?;
        Ok(())
    }
}

pub fn discover() -> Vec<Application> {
    discover_in_roots(&default_application_roots())
}

pub fn discover_in_roots(roots: &[PathBuf]) -> Vec<Application> {
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
            if let Some(application) = application_from_bundle(&path) {
                if seen_paths.insert(application.path.clone()) {
                    applications.push(application);
                }
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
    use std::{fs, path::PathBuf, time::{SystemTime, UNIX_EPOCH}};

    #[test]
    fn discovers_user_launchable_applications_from_bundle_metadata() {
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

        let applications = discover_in_roots(&[root.clone()]);

        assert_eq!(applications.len(), 1);
        assert_eq!(applications[0].name, "Safari Browser");
        assert!(applications[0].path.ends_with("Safari.app"));

        fs::remove_dir_all(root).unwrap();
    }

    fn write_info_plist(bundle_path: &PathBuf, contents: &str) {
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
        std::env::temp_dir().join(format!("launchkick-apps-{nanos}"))
    }
}
