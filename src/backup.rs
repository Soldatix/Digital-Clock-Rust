//! Offline, path-confined JSON snapshots with validation before restore.
use crate::config::{self, P2State};
use std::{fs, path::{Path, PathBuf}};
const MAX_BACKUP_BYTES: u64 = 2 * 1024 * 1024;
fn directory(state_file: &Path) -> Result<PathBuf, String> {
    Ok(state_file.parent().ok_or("Missing Data directory")?.join("Backups"))
}
fn valid_basename(name: &str) -> bool {
    name.starts_with("dc-backup-") && name.ends_with(".json") &&
    name.len() < 120 && name.bytes().all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'.') &&
    !name.contains("..")
}
pub fn list(state_file: &Path) -> Result<Vec<String>, String> {
    let dir=directory(state_file)?;
    if !dir.exists() { return Ok(Vec::new()); }
    let mut names=Vec::new();
    for entry in fs::read_dir(&dir).map_err(|e|e.to_string())? {
        let entry=entry.map_err(|e|e.to_string())?;
        if entry.file_type().map_err(|e|e.to_string())?.is_file() {
            if let Some(name)=entry.file_name().to_str() {
                if valid_basename(name) { names.push(name.to_string()); }
            }
        }
    }
    names.sort_unstable_by(|a,b|b.cmp(a));
    names.truncate(250);
    Ok(names)
}
pub fn export(state_file: &Path, state: &P2State) -> Result<String,String> {
    config::validate(state)?;
    let dir=directory(state_file)?;
    fs::create_dir_all(&dir).map_err(|e|e.to_string())?;
    let nanos=std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH)
        .map_err(|e|e.to_string())?.as_nanos();
    let name=format!("dc-backup-{nanos}-{}.json",std::process::id());
    let target=dir.join(&name);
    if target.exists() { return Err("Backup collision - STOP".into()); }
    config::save(&target,state).map_err(|e|e.to_string())?;
    if fs::metadata(&target).map_err(|e|e.to_string())?.len() > MAX_BACKUP_BYTES {
        let _ = fs::remove_file(&target);
        return Err("Backup larger than 2 MiB".into());
    }
    Ok(name)
}
pub fn inspect(state_file: &Path, name: &str) -> Result<P2State,String> {
    if !valid_basename(name) { return Err("Invalid backup filename".into()); }
    let file=directory(state_file)?.join(name);
    let meta=fs::symlink_metadata(&file).map_err(|e|e.to_string())?;
    if !meta.file_type().is_file() || meta.len() > MAX_BACKUP_BYTES { return Err("Invalid backup file type or size".into()); }
    let raw=fs::read(&file).map_err(|e|e.to_string())?;
    let state:P2State=serde_json::from_slice(&raw).map_err(|e|format!("Backup JSON error: {e}"))?;
    config::validate(&state)?;
    Ok(state)
}

/// Write a validated JSON snapshot to an explicitly chosen outside file.
/// The user confirms overwriting through the native Windows Save As dialog.
pub fn export_external(filename: &Path, state: &P2State) -> Result<(), String> {
    if !is_json_path(filename) { return Err("Select a .json backup file".into()); }
    config::validate(state)?;
    let bytes = serde_json::to_vec_pretty(state).map_err(|e| e.to_string())?;
    if bytes.len() as u64 > MAX_BACKUP_BYTES { return Err("Backup larger than 2 MiB".into()); }
    config::save(filename, state).map_err(|e| e.to_string())
}

pub fn inspect_external(filename: &Path) -> Result<P2State, String> {
    if !is_json_path(filename) { return Err("Select a .json backup file".into()); }
    let meta = fs::symlink_metadata(filename).map_err(|e| e.to_string())?;
    if !meta.file_type().is_file() || meta.len() > MAX_BACKUP_BYTES {
        return Err("Invalid backup file type or size".into());
    }
    let bytes = fs::read(filename).map_err(|e| e.to_string())?;
    let state: P2State = serde_json::from_slice(&bytes).map_err(|e| format!("Backup JSON error: {e}"))?;
    config::validate(&state)?;
    Ok(state)
}

fn is_json_path(path: &Path) -> bool {
    path.extension().and_then(|e| e.to_str()).is_some_and(|ext| ext.eq_ignore_ascii_case("json"))
}

#[cfg(test)] mod tests {
    use super::*;
    #[test] fn round_trip_and_traversal_rejection() {
        let path=std::env::temp_dir().join(format!("dc-p27-backup-test-{}-{}",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()))
            .join("Data").join("app-state-native-p27.json");
        let state=P2State::default();
        let name=export(&path,&state).unwrap();
        assert_eq!(inspect(&path,&name).unwrap(),state);
        assert!(inspect(&path,"../evil.json").is_err());
        assert!(inspect(&path,"C:\\Windows\\x.json").is_err());
        assert_eq!(list(&path).unwrap().len(),1);
        fs::remove_dir_all(path.parent().unwrap().parent().unwrap()).unwrap();
    }
    #[test] fn external_backup_roundtrip_and_invalid_json() {
        let folder = std::env::temp_dir().join(format!("dc-external-test-{}-{}", std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()));
        fs::create_dir_all(&folder).unwrap();
        let filename = folder.join("test.json");
        let original = P2State::default();
        export_external(&filename, &original).unwrap();
        assert_eq!(inspect_external(&filename).unwrap(), original);
        fs::write(&filename, "{").unwrap();
        assert!(inspect_external(&filename).is_err());
        assert!(inspect_external(&folder.join("missing.txt")).is_err());
        fs::remove_dir_all(folder).unwrap();
    }
    #[test] fn corrupted_backup_fail_closed() {
        let path=std::env::temp_dir().join(format!("dc-p27-invalid-{}-{}",std::process::id(),
            std::time::SystemTime::now().duration_since(std::time::UNIX_EPOCH).unwrap().as_nanos()))
            .join("Data").join("app-state-native-p27.json");
        let name=export(&path,&P2State::default()).unwrap();
        let file=directory(&path).unwrap().join(&name);
        fs::write(&file,"{" ).unwrap();
        assert!(inspect(&path,&name).is_err());
        fs::remove_dir_all(path.parent().unwrap().parent().unwrap()).unwrap();
    }
}
