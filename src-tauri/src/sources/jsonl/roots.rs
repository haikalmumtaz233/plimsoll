use std::collections::HashSet;
use std::ffi::OsString;
use std::fs;
use std::path::{Path, PathBuf};

use crate::sources::claude_home_from;

pub const NESTED_DEPTH: usize = 4;

const PROJECTS: &str = "projects";
const CLAUDE_DIR: &str = ".claude";
const XDG_FOLDER: [&str; 2] = [".config", "claude"];
const DESKTOP_FOLDER: &str = "Claude";
const PACKAGE_PREFIX: &str = "Claude_";
const PACKAGE_ROAMING: [&str; 3] = ["LocalCache", "Roaming", DESKTOP_FOLDER];
const PACKAGES: &str = "Packages";

#[must_use]
pub fn discover() -> Vec<PathBuf> {
    let lookup = std::env::var_os;
    let names = package_names(lookup("LOCALAPPDATA").map(PathBuf::from));
    let nested = desktop_bases_from(lookup, &names)
        .iter()
        .flat_map(|base| nested_projects(base))
        .collect::<Vec<_>>();
    let candidates = fixed_candidates_from(lookup)
        .into_iter()
        .chain(nested)
        .collect();
    select(candidates, Path::is_dir)
}

fn package_names(local: Option<PathBuf>) -> Vec<String> {
    let Some(entries) = local.and_then(|local| fs::read_dir(local.join(PACKAGES)).ok()) else {
        return Vec::new();
    };
    entries
        .flatten()
        .filter_map(|entry| entry.file_name().into_string().ok())
        .filter(|name| name.starts_with(PACKAGE_PREFIX))
        .collect()
}

#[must_use]
pub fn fixed_candidates_from<F>(lookup: F) -> Vec<PathBuf>
where
    F: Fn(&'static str) -> Option<OsString>,
{
    let home = claude_home_from(&lookup).map(|home| home.join(PROJECTS));
    let xdg = non_empty(&lookup, "USERPROFILE").map(|profile| {
        XDG_FOLDER
            .iter()
            .fold(profile, |path, part| path.join(part))
            .join(PROJECTS)
    });
    home.into_iter().chain(xdg).collect()
}

#[must_use]
pub fn desktop_bases_from<F>(lookup: F, package_names: &[String]) -> Vec<PathBuf>
where
    F: Fn(&'static str) -> Option<OsString>,
{
    let roaming = non_empty(&lookup, "APPDATA").map(|appdata| appdata.join(DESKTOP_FOLDER));
    let packages = non_empty(&lookup, "LOCALAPPDATA")
        .map(|local| local.join(PACKAGES))
        .into_iter()
        .flat_map(|packages| {
            package_names
                .iter()
                .filter(|name| name.starts_with(PACKAGE_PREFIX))
                .map(move |name| {
                    PACKAGE_ROAMING
                        .iter()
                        .fold(packages.join(name), |path, part| path.join(part))
                })
        });
    roaming.into_iter().chain(packages).collect()
}

fn non_empty<F>(lookup: &F, name: &'static str) -> Option<PathBuf>
where
    F: Fn(&'static str) -> Option<OsString>,
{
    lookup(name)
        .filter(|value| !value.is_empty())
        .map(PathBuf::from)
}

#[must_use]
pub fn select<F>(candidates: Vec<PathBuf>, is_dir: F) -> Vec<PathBuf>
where
    F: Fn(&Path) -> bool,
{
    let mut seen = HashSet::new();
    candidates
        .into_iter()
        .filter(|candidate| is_dir(candidate))
        .filter(|candidate| seen.insert(candidate.to_string_lossy().to_lowercase()))
        .collect()
}

#[must_use]
pub fn nested_projects(base: &Path) -> Vec<PathBuf> {
    let mut found = Vec::new();
    walk(base, 0, &mut found);
    found.sort();
    found
}

fn walk(directory: &Path, depth: usize, found: &mut Vec<PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    let child_depth = depth + 1;
    for entry in entries.flatten() {
        if !entry.file_type().is_ok_and(|kind| kind.is_dir()) {
            continue;
        }
        let path = entry.path();
        if entry.file_name() == CLAUDE_DIR {
            let projects = path.join(PROJECTS);
            if projects.is_dir() {
                found.push(projects);
            }
        } else if child_depth < NESTED_DEPTH {
            walk(&path, child_depth, found);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::{desktop_bases_from, fixed_candidates_from, nested_projects, select};
    use crate::sources::test_env::lookup;
    use std::fs;
    use std::path::{Path, PathBuf};

    #[test]
    fn fixed_candidates_cover_the_claude_home_and_the_xdg_folder() {
        assert_eq!(
            fixed_candidates_from(lookup(&[("USERPROFILE", r"C:\Users\dev")])),
            vec![
                PathBuf::from(r"C:\Users\dev\.claude\projects"),
                PathBuf::from(r"C:\Users\dev\.config\claude\projects"),
            ]
        );
        assert_eq!(
            fixed_candidates_from(lookup(&[
                ("USERPROFILE", r"C:\Users\dev"),
                ("CLAUDE_CONFIG_DIR", r"D:\claude"),
            ])),
            vec![
                PathBuf::from(r"D:\claude\projects"),
                PathBuf::from(r"C:\Users\dev\.config\claude\projects"),
            ]
        );
        assert!(fixed_candidates_from(lookup(&[])).is_empty());
    }

    #[test]
    fn desktop_bases_include_the_packaged_app_data() {
        let names = vec![
            "Claude_pzs8sxrjxfjjc".to_owned(),
            "Microsoft.Paint_8wekyb3d8bbwe".to_owned(),
            "ClaudeHelper".to_owned(),
        ];
        assert_eq!(
            desktop_bases_from(
                lookup(&[
                    ("APPDATA", r"C:\Users\dev\AppData\Roaming"),
                    ("LOCALAPPDATA", r"C:\Users\dev\AppData\Local"),
                ]),
                &names
            ),
            vec![
                PathBuf::from(r"C:\Users\dev\AppData\Roaming\Claude"),
                PathBuf::from(
                    r"C:\Users\dev\AppData\Local\Packages\Claude_pzs8sxrjxfjjc\LocalCache\Roaming\Claude"
                ),
            ]
        );
        assert!(desktop_bases_from(lookup(&[]), &names).is_empty());
    }

    #[test]
    fn selection_keeps_existing_folders_once_in_order() {
        let candidates = vec![
            PathBuf::from(r"C:\Users\dev\.claude\projects"),
            PathBuf::from(r"C:\Users\dev\.config\claude\projects"),
            PathBuf::from(r"c:\users\DEV\.claude\projects"),
            PathBuf::from(r"D:\other\projects"),
        ];
        let existing = |path: &Path| !path.starts_with(r"C:\Users\dev\.config");
        assert_eq!(
            select(candidates, existing),
            vec![
                PathBuf::from(r"C:\Users\dev\.claude\projects"),
                PathBuf::from(r"D:\other\projects"),
            ]
        );
    }

    #[test]
    fn finds_nested_claude_projects_within_the_depth_limit() {
        let base = std::env::temp_dir().join(format!("plimsoll-roots-{}", std::process::id()));
        fs::remove_dir_all(&base).ok();
        let shallow = base
            .join("sessions")
            .join("a")
            .join(".claude")
            .join("projects");
        let deep = base
            .join("1")
            .join("2")
            .join("3")
            .join("4")
            .join(".claude")
            .join("projects");
        let empty = base.join("sessions").join("b").join(".claude");
        for directory in [&shallow, &deep, &empty] {
            fs::create_dir_all(directory).expect("create");
        }
        assert_eq!(nested_projects(&base), vec![shallow]);
        assert!(nested_projects(&base.join("missing")).is_empty());
        fs::remove_dir_all(base).expect("cleanup");
    }
}
