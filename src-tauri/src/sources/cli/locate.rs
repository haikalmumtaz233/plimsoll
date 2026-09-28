use std::ffi::OsStr;
use std::path::{Path, PathBuf};

pub const EXECUTABLES: [&str; 2] = ["claude.exe", "claude.cmd"];

const NATIVE_FOLDER: [&str; 2] = [".local", "bin"];

#[must_use]
pub fn locate<F>(path_var: Option<&OsStr>, home: Option<&Path>, exists: F) -> Option<PathBuf>
where
    F: Fn(&Path) -> bool,
{
    let listed = path_var
        .into_iter()
        .flat_map(std::env::split_paths)
        .filter(|directory| directory.is_absolute());
    let native = home.map(|home| home.join(NATIVE_FOLDER[0]).join(NATIVE_FOLDER[1]));
    listed
        .chain(native)
        .flat_map(|directory| EXECUTABLES.map(|name| directory.join(name)))
        .find(|candidate| exists(candidate))
}

#[cfg(test)]
mod tests {
    use super::locate;
    use std::ffi::OsString;
    use std::path::{Path, PathBuf};

    fn path_var(entries: &[&str]) -> OsString {
        std::env::join_paths(entries).expect("joinable path")
    }

    fn existing(files: &'static [&'static str]) -> impl Fn(&Path) -> bool {
        move |path| files.iter().any(|file| Path::new(file) == path)
    }

    #[test]
    fn finds_the_native_executable_on_the_path() {
        let path = path_var(&[r"C:\Windows", r"C:\Tools"]);
        assert_eq!(
            locate(Some(&path), None, existing(&[r"C:\Tools\claude.exe"])),
            Some(PathBuf::from(r"C:\Tools\claude.exe"))
        );
    }

    #[test]
    fn prefers_the_earliest_directory_then_the_exe_over_the_script() {
        let path = path_var(&[r"C:\npm", r"C:\Tools"]);
        assert_eq!(
            locate(
                Some(&path),
                None,
                existing(&[r"C:\npm\claude.cmd", r"C:\Tools\claude.exe"])
            ),
            Some(PathBuf::from(r"C:\npm\claude.cmd"))
        );
        assert_eq!(
            locate(
                Some(&path),
                None,
                existing(&[r"C:\npm\claude.cmd", r"C:\npm\claude.exe"])
            ),
            Some(PathBuf::from(r"C:\npm\claude.exe"))
        );
    }

    #[test]
    fn falls_back_to_the_native_install_folder() {
        let home = Path::new(r"C:\Users\dev");
        assert_eq!(
            locate(
                None,
                Some(home),
                existing(&[r"C:\Users\dev\.local\bin\claude.exe"])
            ),
            Some(PathBuf::from(r"C:\Users\dev\.local\bin\claude.exe"))
        );
    }

    #[test]
    fn ignores_relative_path_entries() {
        let path = path_var(&[".", r"tools"]);
        assert_eq!(
            locate(
                Some(&path),
                None,
                existing(&[r".\claude.exe", r"tools\claude.exe"])
            ),
            None
        );
    }

    #[test]
    fn nothing_found_is_none() {
        let path = path_var(&[r"C:\Windows"]);
        assert_eq!(
            locate(Some(&path), Some(Path::new(r"C:\Users\dev")), |_| false),
            None
        );
    }
}
