use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};
use tokio::sync::Mutex;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Snapshot {
    pub id: String,
    pub created_at: String,
    pub input: serde_json::Value,
    pub output: serde_json::Value,
}

pub struct SnapshotStore {
    path: PathBuf,
    snapshots: Mutex<Vec<Snapshot>>,
}

impl SnapshotStore {
    pub async fn load(path: PathBuf) -> Result<Self, String> {
        let snapshots = if path.exists() {
            let data = tokio::fs::read_to_string(&path)
                .await
                .map_err(|err| format!("failed to read snapshots: {}", err))?;
            if data.trim().is_empty() {
                Vec::new()
            } else {
                serde_json::from_str(&data)
                    .map_err(|err| format!("failed to parse snapshots: {}", err))?
            }
        } else {
            Vec::new()
        };

        Ok(Self {
            path,
            snapshots: Mutex::new(snapshots),
        })
    }

    pub async fn list(&self) -> Vec<Snapshot> {
        let guard = self.snapshots.lock().await;
        guard.clone()
    }

    pub async fn get(&self, snapshot_id: &str) -> Option<Snapshot> {
        let guard = self.snapshots.lock().await;
        guard.iter().find(|snapshot| snapshot.id == snapshot_id).cloned()
    }

    pub async fn add(&self, snapshot: Snapshot) -> Result<Snapshot, String> {
        let mut guard = self.snapshots.lock().await;
        guard.insert(0, snapshot.clone());
        if guard.len() > 50 {
            guard.truncate(50);
        }
        self.persist(&guard).await?;
        Ok(snapshot)
    }

    pub async fn delete(&self, snapshot_id: &str) -> Result<bool, String> {
        let mut guard = self.snapshots.lock().await;
        let before = guard.len();
        guard.retain(|snapshot| snapshot.id != snapshot_id);
        let removed = guard.len() != before;
        if removed {
            self.persist(&guard).await?;
        }
        Ok(removed)
    }

    async fn persist(&self, snapshots: &[Snapshot]) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            ensure_dir(parent).await?;
        }
        let payload = serde_json::to_string_pretty(snapshots)
            .map_err(|err| format!("failed to serialize snapshots: {}", err))?;
        let tmp_path = self.path.with_extension("json.tmp");
        tokio::fs::write(&tmp_path, payload)
            .await
            .map_err(|err| format!("failed to write snapshots: {}", err))?;
        tokio::fs::rename(&tmp_path, &self.path)
            .await
            .map_err(|err| format!("failed to finalize snapshots: {}", err))?;
        Ok(())
    }
}

async fn ensure_dir(path: &Path) -> Result<(), String> {
    if path.exists() {
        return Ok(());
    }
    tokio::fs::create_dir_all(path)
        .await
        .map_err(|err| format!("failed to create snapshot dir: {}", err))
}
