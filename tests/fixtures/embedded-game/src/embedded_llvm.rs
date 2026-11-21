use mixin_runtime::{Result, bundle::sha256};
use std::fs::{self, File, OpenOptions};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::{SystemTime, UNIX_EPOCH};

const LLVM: &[u8] = include_bytes!("../../../../.tools/llvm-21.1.3/bin/LLVM-C.dll");
pub const LICENSE: &str = include_str!("../../../../.tools/llvm-21.1.3/LLVM-LICENSE.txt");

pub fn install() -> Result<PathBuf> {
    let data = std::env::var_os("LOCALAPPDATA")
        .map(PathBuf::from)
        .filter(|path| path.is_absolute())
        .ok_or("LOCALAPPDATA must be set to an absolute directory for embedded LLVM")?;
    let directory = data.join("RustMixin").join("llvm").join(sha256(LLVM));
    install_in(&directory).map_err(|error| format!("{}: {error}", directory.display()))
}

fn install_in(directory: &Path) -> io::Result<PathBuf> {
    fs::create_dir_all(directory)?;
    let dll = directory.join("LLVM-C.dll");
    if matches_embedded(&dll)? {
        return Ok(dll);
    }

    let time = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_nanos();
    let staging = directory.join(format!("llvm-{}-{time}.tmp", std::process::id()));
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(&staging)?;
    let written = file.write_all(LLVM);
    drop(file);
    let installed = written.and_then(|()| {
        // Publish the finished file in one rename. Another process may already
        // have published and loaded an identical DLL.
        match fs::rename(&staging, &dll) {
            Err(_) if matches_embedded(&dll)? => Ok(()),
            result => result,
        }
    });
    let _ = fs::remove_file(&staging);
    installed?;
    Ok(dll)
}

fn matches_embedded(path: &Path) -> io::Result<bool> {
    let file = match File::open(path) {
        Ok(file) => file,
        Err(error) if error.kind() == io::ErrorKind::NotFound => return Ok(false),
        Err(error) => return Err(error),
    };
    if file.metadata()?.len() != LLVM.len() as u64 {
        return Ok(false);
    }
    let mut bytes = Vec::with_capacity(LLVM.len());
    file.take(LLVM.len() as u64 + 1).read_to_end(&mut bytes)?;
    Ok(bytes == LLVM)
}
