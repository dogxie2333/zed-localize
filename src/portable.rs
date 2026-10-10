//! Single-file distribution: extract bundled dependencies once beside the EXE.
use anyhow::{Context, Result, ensure};
use std::{
    fs,
    io::{Read, Seek, SeekFrom, copy},
    path::PathBuf,
};
const MAGIC: &[u8] = b"ZED_ZH_PAYLOAD_V1";
struct PayloadFile {
    file: fs::File,
    start: u64,
    length: u64,
    position: u64,
}
impl Read for PayloadFile {
    fn read(&mut self, buffer: &mut [u8]) -> std::io::Result<usize> {
        let amount = (self.length - self.position).min(buffer.len() as u64) as usize;
        let read = self.file.read(&mut buffer[..amount])?;
        self.position += read as u64;
        Ok(read)
    }
}
impl Seek for PayloadFile {
    fn seek(&mut self, from: SeekFrom) -> std::io::Result<u64> {
        let position = match from {
            SeekFrom::Start(n) => i128::from(n),
            SeekFrom::Current(n) => i128::from(self.position) + i128::from(n),
            SeekFrom::End(n) => i128::from(self.length) + i128::from(n),
        };
        if position < 0 || position > i128::from(self.length) {
            return Err(std::io::Error::new(
                std::io::ErrorKind::InvalidInput,
                "Invalid payload seek",
            ));
        }
        self.position = position as u64;
        self.file
            .seek(SeekFrom::Start(self.start + self.position))?;
        Ok(self.position)
    }
}

pub fn root() -> Result<PathBuf> {
    let executable = std::env::current_exe()?;
    let mut file = fs::File::open(&executable)?;
    let size = file.metadata()?.len();
    let footer_size = (8 + 64 + MAGIC.len()) as u64;
    ensure!(size > footer_size, "Missing portable payload");
    file.seek(SeekFrom::End(-(footer_size as i64)))?;
    let mut footer = vec![0u8; footer_size as usize];
    file.read_exact(&mut footer)?;
    ensure!(
        &footer[72..] == MAGIC,
        "Missing portable payload; use packaged EXE"
    );
    let payload_length = u64::from_le_bytes(footer[..8].try_into()?);
    ensure!(
        payload_length < size - footer_size,
        "Invalid payload length"
    );
    let payload_hash = std::str::from_utf8(&footer[8..72])?.to_owned();
    let start = size - footer_size - payload_length;
    file.seek(SeekFrom::Start(start))?;
    let reader = PayloadFile {
        file,
        start,
        length: payload_length,
        position: 0,
    };
    let parent = executable.parent().context("No EXE directory")?;
    let state = if let Some(path) = std::env::var_os("ZED_ZH_STATE_DIR") {
        PathBuf::from(path)
    } else if parent.file_name().is_some_and(|name| name == "bin")
        && parent
            .parent()
            .is_some_and(|path| path.join("portable-state.json").is_file())
    {
        parent.parent().unwrap().to_owned()
    } else {
        parent.join("zed-zh-data")
    };
    fs::create_dir_all(&state)?;
    let _ = crate::runtime_paths::STATE.set(state.clone());
    let resource_root = state.join("runtime").join(&payload_hash[..16]);
    if !resource_root.join("bundle-ready.json").is_file() {
        let staging = state
            .join("runtime")
            .join(format!("staging-{}", std::process::id()));
        fs::create_dir_all(&staging)?;
        let mut archive = zip::ZipArchive::new(reader)?;
        for index in 0..archive.len() {
            let mut entry = archive.by_index(index)?;
            let name = entry.enclosed_name().context("Invalid bundle path")?;
            let destination = staging.join(name);
            ensure!(
                destination.starts_with(&staging),
                "Bundle path escaped cache"
            );
            if entry.is_dir() {
                fs::create_dir_all(destination)?;
                continue;
            }
            fs::create_dir_all(destination.parent().context("Missing bundle directory")?)?;
            copy(&mut entry, &mut fs::File::create(destination)?)?;
        }
        fs::write(
            staging.join("bundle-ready.json"),
            format!("{{\"sha256\":\"{payload_hash}\"}}"),
        )?;
        fs::create_dir_all(resource_root.parent().unwrap())?;
        match fs::rename(&staging, &resource_root) {
            Ok(()) => {}
            Err(error) if resource_root.join("bundle-ready.json").is_file() => {
                let _ = error;
            }
            Err(error) => return Err(error.into()),
        }
    }
    let stable = state.join("bin/zed-zh-launcher.exe");
    fs::create_dir_all(stable.parent().unwrap())?;
    let version = state.join("frontend-version.txt");
    if fs::read_to_string(&version).ok().as_deref() != Some(payload_hash.as_str())
        && executable != stable
    {
        if stable.exists() {
            fs::write(
                state.join("tray-stop.json"),
                serde_json::to_vec(
                    &serde_json::json!({"time":std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)?.as_secs_f64()}),
                )?,
            )?;
            for _ in 0..20 {
                if fs::OpenOptions::new().write(true).open(&stable).is_ok() {
                    break;
                }
                std::thread::sleep(std::time::Duration::from_millis(250));
            }
            // This is only the managed application executable, never user data.
            fs::remove_file(&stable)?;
        }
        if fs::hard_link(&executable, &stable).is_err() {
            fs::copy(&executable, &stable)?;
        }
        fs::write(&version, &payload_hash)?;
    }
    fs::write(state.join("portable-state.json"), "{\"schema\":1}")?;
    Ok(resource_root)
}
