//! Bounded reads for optional repository-controlled configuration files.

use std::fs::OpenOptions;
use std::io::Read;
use std::path::Path;

use anyhow::{bail, Context, Result};

const MAX_CONFIG_BYTES: u64 = 1024 * 1024;

pub(crate) fn read_regular_text(path: &Path) -> Result<String> {
    read_regular_text_with_limit(path, MAX_CONFIG_BYTES as usize)
}

pub(crate) fn read_regular_text_with_limit(path: &Path, max_bytes: usize) -> Result<String> {
    let mut options = OpenOptions::new();
    options.read(true);
    #[cfg(unix)]
    {
        use std::os::unix::fs::OpenOptionsExt;
        options.custom_flags(libc::O_NONBLOCK);
    }
    let file = options
        .open(path)
        .with_context(|| format!("Cannot open '{}'", path.display()))?;
    let metadata = file
        .metadata()
        .with_context(|| format!("Cannot inspect '{}'", path.display()))?;
    if !metadata.is_file() {
        bail!("'{}' is not a regular file", path.display());
    }
    if metadata.len() > max_bytes as u64 {
        bail!("'{}' exceeds the {max_bytes}-byte limit", path.display());
    }
    let mut bytes = Vec::new();
    file.take(max_bytes as u64 + 1)
        .read_to_end(&mut bytes)
        .with_context(|| format!("Cannot read '{}'", path.display()))?;
    if bytes.len() > max_bytes {
        bail!("'{}' exceeds the {max_bytes}-byte limit", path.display());
    }
    String::from_utf8(bytes).with_context(|| format!("'{}' is not UTF-8", path.display()))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_large_and_special_files() {
        let path = std::env::temp_dir().join(format!(
            "droast-safe-file-{}-{}",
            std::process::id(),
            std::thread::current().name().unwrap_or("test")
        ));
        std::fs::write(&path, "ok").unwrap();
        assert_eq!(read_regular_text(&path).unwrap(), "ok");
        std::fs::write(&path, vec![b'x'; MAX_CONFIG_BYTES as usize + 1]).unwrap();
        assert!(read_regular_text(&path).is_err());
        std::fs::remove_file(&path).unwrap();

        #[cfg(unix)]
        assert!(read_regular_text(Path::new("/dev/zero")).is_err());

        #[cfg(target_os = "linux")]
        {
            let status = std::process::Command::new("mkfifo")
                .arg(&path)
                .status()
                .unwrap();
            assert!(status.success());
            assert!(read_regular_text(&path).is_err());
            std::fs::remove_file(&path).unwrap();
        }
    }
}
