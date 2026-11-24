//! Write a complete static site. The build driver supplies generated UTF-8 files;
//! native Rust copies the static assets and owns publication to the local dist.
use std::{
    collections::HashSet,
    fs,
    io::{self, Read},
    path::{Path, PathBuf},
    process::ExitCode,
};

const LIMIT: u64 = 64 * 1024 * 1024;
const INVALID: &str = "無效的網站產物資料";
type Result<T> = std::result::Result<T, String>;

fn valid_path(path: &str) -> bool {
    !path.is_empty()
        && path.split('/').all(|part| {
            !part.is_empty()
                && !matches!(part, "." | "..")
                && !part.ends_with('.')
                && part
                    .bytes()
                    .all(|b| b.is_ascii_alphanumeric() || b"_.-".contains(&b))
        })
}

fn decode(input: &str) -> Result<Vec<(&str, &str)>> {
    let invalid = || INVALID.to_owned();
    let mut fields = input.strip_suffix('\0').ok_or_else(invalid)?.split('\0');
    if fields.next() != Some("site-write-v1") {
        return Err(invalid());
    }
    let count = fields
        .next()
        .filter(|s| !s.is_empty() && s.bytes().all(|b| b.is_ascii_digit()))
        .and_then(|s| s.parse::<usize>().ok())
        .filter(|n| *n <= 1000)
        .ok_or_else(invalid)?;
    let mut files = Vec::with_capacity(count);
    let mut seen = HashSet::new();
    for _ in 0..count {
        let path = fields.next().ok_or_else(invalid)?;
        let text = fields.next().ok_or_else(invalid)?;
        if !valid_path(path) || !seen.insert(path) {
            return Err(invalid());
        }
        files.push((path, text));
    }
    if fields.next().is_some()
        || files.iter().any(|(path, _)| {
            path.match_indices('/')
                .any(|(i, _)| seen.contains(&path[..i]))
        })
    {
        return Err(invalid());
    }
    Ok(files)
}

fn ordinary_dir(path: &Path) -> io::Result<()> {
    if fs::symlink_metadata(path)?.is_dir() {
        Ok(())
    } else {
        Err(io::Error::new(
            io::ErrorKind::InvalidInput,
            "not a regular directory",
        ))
    }
}

fn copy_tree(from: &Path, to: &Path) -> io::Result<()> {
    ordinary_dir(from)?;
    fs::create_dir_all(to)?;
    for entry in fs::read_dir(from)? {
        let entry = entry?;
        let ty = entry.file_type()?;
        let target = to.join(entry.file_name());
        if ty.is_dir() {
            copy_tree(&entry.path(), &target)?;
        } else if ty.is_file() {
            fs::copy(entry.path(), target)?;
        } else {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "unsupported asset type",
            ));
        }
    }
    Ok(())
}

struct Stage(PathBuf);
impl Drop for Stage {
    fn drop(&mut self) {
        let _ = fs::remove_dir_all(&self.0);
    }
}

fn stage(root: &Path) -> io::Result<Stage> {
    for n in 0..100 {
        let path = root.join(format!(".site-write-{}-{n}", std::process::id()));
        match fs::create_dir(&path) {
            Ok(()) => return Ok(Stage(path)),
            Err(err) if err.kind() == io::ErrorKind::AlreadyExists => continue,
            Err(err) => return Err(err),
        }
    }
    Err(io::Error::new(
        io::ErrorKind::AlreadyExists,
        "no staging directory available",
    ))
}

fn write_site(root: &Path, files: &[(&str, &str)]) -> io::Result<()> {
    let dist = root.join("dist");
    ordinary_dir(&root.join("src"))?;
    ordinary_dir(&root.join("src/assets"))?;
    match fs::symlink_metadata(&dist) {
        Ok(meta) if !meta.is_dir() => {
            return Err(io::Error::new(
                io::ErrorKind::InvalidInput,
                "dist must be an ordinary directory",
            ));
        }
        Ok(_) => {}
        Err(err) if err.kind() == io::ErrorKind::NotFound => {}
        Err(err) => return Err(err),
    }
    let temp = stage(root)?;
    let output = temp.0.join("output");
    copy_tree(&root.join("src/assets"), &output.join("assets"))?;
    for (path, text) in files {
        let file = output.join(path);
        fs::create_dir_all(
            file.parent()
                .ok_or_else(|| io::Error::other("missing parent"))?,
        )?;
        fs::write(file, text.as_bytes())?;
    }
    // Preserve the previous dist until every new file has been staged. Keep a
    // backup while swapping, and restore it if the final rename fails.
    let backup = temp.0.join("previous");
    let had_previous = dist.exists();
    if had_previous {
        fs::rename(&dist, &backup)?;
    }
    if let Err(err) = fs::rename(&output, &dist) {
        if had_previous {
            if let Err(restore) = fs::rename(&backup, &dist) {
                // Leave the backup on disk rather than delete the old site.
                let path = temp.0.clone();
                std::mem::forget(temp);
                return Err(io::Error::other(format!(
                    "{err}; restore failed: {restore}; backup: {}",
                    path.display()
                )));
            }
        }
        return Err(err);
    }
    Ok(())
}

fn run() -> Result<()> {
    let mut input = String::new();
    io::stdin()
        .lock()
        .take(LIMIT + 1)
        .read_to_string(&mut input)
        .map_err(|_| INVALID.to_owned())?;
    if input.len() as u64 > LIMIT {
        return Err(INVALID.into());
    }
    let files = decode(&input)?;
    let root = std::env::current_dir().map_err(|_| "無法寫入網站產物".to_owned())?;
    write_site(&root, &files).map_err(|err| {
        eprintln!("site-write I/O: {err}");
        "無法寫入網站產物".into()
    })
}

fn main() -> ExitCode {
    match run() {
        Ok(()) => ExitCode::SUCCESS,
        Err(err) => {
            eprintln!("{err}");
            ExitCode::FAILURE
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn protocol_preserves_unicode_and_empty_files() {
        assert_eq!(
            decode("site-write-v1\0\x32\0index.html\0教材🦀\0.nojekyll\0\0").unwrap(),
            vec![("index.html", "教材🦀"), (".nojekyll", "")]
        );
    }

    #[test]
    fn rejects_traversal_duplicates_conflicts_and_truncation() {
        for input in [
            "site-write-v1\0\x31\0../outside\0bad\0",
            "site-write-v1\0\x32\0a\0x\0a\0y\0",
            "site-write-v1\0\x32\0a\0x\0a/b\0y\0",
            "site-write-v1\0\x31\0a\0",
            "site-write-v1\0\x30\0extra\0",
            "old\0\x30\0",
        ] {
            assert!(decode(input).is_err(), "{input:?}");
        }
    }

    #[test]
    fn writes_nested_output_copies_binary_assets_and_removes_stale_files() {
        let root = stage(&std::env::temp_dir()).unwrap();
        fs::create_dir_all(root.0.join("src/assets/nested")).unwrap();
        fs::write(root.0.join("src/assets/nested/icon.bin"), [0, 255, 1]).unwrap();
        fs::create_dir(root.0.join("dist")).unwrap();
        fs::write(root.0.join("dist/stale.html"), "old").unwrap();
        write_site(
            &root.0,
            &[("chapters/a/index.html", "Rust"), (".nojekyll", "")],
        )
        .unwrap();
        assert_eq!(
            fs::read(root.0.join("dist/assets/nested/icon.bin")).unwrap(),
            [0, 255, 1]
        );
        assert_eq!(
            fs::read_to_string(root.0.join("dist/chapters/a/index.html")).unwrap(),
            "Rust"
        );
        assert!(!root.0.join("dist/stale.html").exists());
        // A collision must leave the last successful build intact.
        assert!(write_site(&root.0, &[("assets/nested", "collision")]).is_err());
        assert!(root.0.join("dist/chapters/a/index.html").is_file());
    }

    #[cfg(unix)]
    #[test]
    fn rejects_asset_and_dist_symlinks() {
        use std::os::unix::fs::symlink;
        let root = stage(&std::env::temp_dir()).unwrap();
        fs::create_dir_all(root.0.join("src/assets")).unwrap();
        fs::create_dir(root.0.join("outside")).unwrap();
        fs::write(root.0.join("outside/keep"), "untouched").unwrap();
        symlink(root.0.join("outside"), root.0.join("dist")).unwrap();
        assert!(write_site(&root.0, &[]).is_err());
        fs::remove_file(root.0.join("dist")).unwrap();
        symlink(root.0.join("outside"), root.0.join("src/assets/escape")).unwrap();
        assert!(write_site(&root.0, &[]).is_err());
        assert_eq!(
            fs::read_to_string(root.0.join("outside/keep")).unwrap(),
            "untouched"
        );
    }
}
