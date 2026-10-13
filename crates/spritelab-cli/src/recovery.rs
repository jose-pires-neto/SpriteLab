//! Recovery snapshots are runtime file effects, separate from saved baselines and undo.
use super::*;
use serde::Serialize;
use std::collections::BTreeMap;
use std::time::{SystemTime, UNIX_EPOCH};
const DIRECTORY: &str = ".spritelab-recovery";
const MAX_SNAPSHOTS: usize = 128;
#[derive(Deserialize, JsonSchema)]
#[serde(deny_unknown_fields)]
struct EntryArgs {
    entry: String,
}
#[derive(Deserialize, Serialize)]
#[serde(deny_unknown_fields)]
struct Snapshot {
    version: u32,
    document_id: String,
    source_path: Option<String>,
    revision: u64,
    updated_ms: u128,
    document: Document,
}
pub(super) struct Recovery {
    namespace: String,
    snapshots: BTreeMap<String, (String, Document)>,
    sequence: u64,
}
impl Recovery {
    pub fn new() -> Self {
        static SEQUENCE: AtomicU64 = AtomicU64::new(0);
        Self {
            namespace: format!(
                "{}-{}-{}",
                now(),
                std::process::id(),
                SEQUENCE.fetch_add(1, Ordering::Relaxed)
            ),
            snapshots: BTreeMap::new(),
            sequence: 0,
        }
    }
}
fn now() -> u128 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_millis()
}
pub(super) fn catalog() -> Vec<Descriptor> {
    [
        (
            "recovery.checkpoint",
            "Write a recovery snapshot without marking the project saved",
            "file_write",
            false,
        ),
        (
            "recovery.list",
            "List bounded recovery snapshots from previous sessions",
            "query",
            false,
        ),
        (
            "recovery.restore",
            "Restore a recovery snapshot into a new unsaved document",
            "lifecycle",
            true,
        ),
        (
            "recovery.delete",
            "Explicitly discard one recovery snapshot",
            "file_write",
            true,
        ),
    ]
    .into_iter()
    .map(|(id, summary, effect, entry)| Descriptor {
        id: id.into(),
        summary: summary.into(),
        effect: effect.into(),
        undoable: false,
        arguments_schema: if entry {
            serde_json::to_value(schemars::schema_for!(EntryArgs)).unwrap()
        } else {
            serde_json::to_value(schemars::schema_for!(Empty)).unwrap()
        },
    })
    .collect()
}
impl Runtime {
    fn recovery_directory(&self) -> Result<PathBuf> {
        let path = self.root.join(DIRECTORY);
        if !path.exists() {
            fs::create_dir(&path).map_err(io_error)?;
        }
        if path
            .symlink_metadata()
            .map_err(io_error)?
            .file_type()
            .is_symlink()
            || !path.is_dir()
        {
            return Err(CommandError::new(
                "path_denied",
                "Recovery directory must be a real directory",
            ));
        }
        Ok(path)
    }
    fn recovery_entry(&self, entry: &str, write: bool) -> Result<PathBuf> {
        if !entry.starts_with("snapshot-")
            || !entry.ends_with(".json")
            || !entry
                .bytes()
                .all(|b| b.is_ascii_alphanumeric() || b == b'-' || b == b'.')
        {
            return Err(invalid("Invalid recovery entry"));
        }
        self.path(&format!("{DIRECTORY}/{entry}"), write)
    }
    pub(super) fn dispatch_recovery(&mut self, r: &Request) -> Result<Value> {
        if r.command == "recovery.list" && !self.root.join(DIRECTORY).exists() {
            decode::<Empty>(r.args.clone())?;
            return Ok(json!({"entries":[]}));
        }
        let directory = self.recovery_directory()?;
        match r.command.as_str() {
            "recovery.list" => {
                decode::<Empty>(r.args.clone())?;
                let mut entries = Vec::new();
                for entry in fs::read_dir(directory).map_err(io_error)? {
                    let entry = entry.map_err(io_error)?;
                    let name = entry.file_name().to_string_lossy().to_string();
                    if !name.starts_with("snapshot-") || !name.ends_with(".json") {
                        continue;
                    }
                    if entries.len() >= MAX_SNAPSHOTS {
                        return Err(CommandError::new(
                            "resource_limit",
                            "Recovery listing exceeds 128 snapshots",
                        ));
                    }
                    // Keep malformed snapshots visible; never silently omit damaged work.
                    let parsed = self.read(&format!("{DIRECTORY}/{name}")).and_then(|bytes| {
                        serde_json::from_slice::<Snapshot>(&bytes)
                            .map_err(|e| invalid(e.to_string()))
                    });
                    entries.push(match parsed {
                        Ok(s) => json!({"entry":name,"document_id":s.document_id,"source_path":s.source_path,"revision":s.revision,"updated_ms":s.updated_ms,"width":s.document.width,"height":s.document.height}),
                        Err(e) => json!({"entry":name,"error":e.message}),
                    });
                }
                entries.sort_by(|a, b| b["updated_ms"].as_u64().cmp(&a["updated_ms"].as_u64()));
                Ok(json!({"entries":entries}))
            }
            "recovery.checkpoint" => {
                decode::<Empty>(r.args.clone())?;
                let id = r
                    .document_id
                    .as_deref()
                    .ok_or_else(|| invalid("document_id required"))?;
                self.engine.check_revision(id, r.expected_revision)?;
                let document = self.engine.document(id)?.clone();
                let existing = self.recovery.snapshots.get(id);
                if let Some((entry, last)) = existing {
                    if last == &document && directory.join(entry).is_file() {
                        return Ok(json!({"entry":entry,"changed":false}));
                    }
                }
                let entry = if let Some((entry, _)) = existing {
                    entry.clone()
                } else {
                    if fs::read_dir(&directory).map_err(io_error)?.count() >= MAX_SNAPSHOTS {
                        return Err(CommandError::new(
                            "resource_limit",
                            "Recovery storage full; explicitly discard old snapshots",
                        ));
                    }
                    self.recovery.sequence += 1;
                    format!(
                        "snapshot-{}-{}.json",
                        self.recovery.namespace, self.recovery.sequence
                    )
                };
                let snapshot = Snapshot {
                    version: 1,
                    document_id: id.into(),
                    source_path: self.saved.get(id).map(|(path, _)| path.clone()),
                    revision: self.engine.revision(id).unwrap(),
                    updated_ms: now(),
                    document: document.clone(),
                };
                let bytes = serde_json::to_vec(&snapshot).map_err(|e| invalid(e.to_string()))?;
                if bytes.len() > MAX_INPUT {
                    return Err(CommandError::new(
                        "resource_limit",
                        "Recovery snapshot exceeds 16 MiB",
                    ));
                }
                atomic_write(
                    &self.recovery_entry(&entry, true)?,
                    &bytes,
                    existing.is_some(),
                )?;
                self.recovery
                    .snapshots
                    .insert(id.into(), (entry.clone(), document));
                Ok(json!({"entry":entry,"changed":true}))
            }
            "recovery.restore" => {
                if r.expected_revision.is_some() {
                    return Err(invalid("Restore must omit expected_revision"));
                }
                let a: EntryArgs = decode(r.args.clone())?;
                self.recovery_entry(&a.entry, false)?;
                let bytes = self.read(&format!("{DIRECTORY}/{}", a.entry))?;
                let snapshot: Snapshot =
                    serde_json::from_slice(&bytes).map_err(|e| invalid(e.to_string()))?;
                if snapshot.version != 1 {
                    return Err(invalid("Unsupported recovery version"));
                }
                let id = r
                    .document_id
                    .as_deref()
                    .ok_or_else(|| invalid("document_id required"))?;
                self.engine.install(id, snapshot.document)
            }
            "recovery.delete" => {
                if r.expected_revision.is_some() {
                    return Err(invalid("Discard must omit expected_revision"));
                }
                let a: EntryArgs = decode(r.args.clone())?;
                let path = self.recovery_entry(&a.entry, true)?;
                fs::remove_file(path).map_err(io_error)?;
                self.recovery
                    .snapshots
                    .retain(|_, (entry, _)| entry != &a.entry);
                Ok(json!({"deleted":a.entry}))
            }
            _ => Err(invalid("Unknown recovery command")),
        }
    }
}
