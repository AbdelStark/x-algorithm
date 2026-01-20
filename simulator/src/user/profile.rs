use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::path::{Path, PathBuf};
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserProfile {
    pub user_id: String,
    pub followers: u64,
    pub following: u64,
    pub account_age_days: u32,
    pub verified: bool,
    pub engagement_history: Vec<EngagementEvent>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EngagementEvent {
    pub post_id: String,
    pub author_id: String,
    pub timestamp: i64,
    pub actions: ActionFlags,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ActionFlags {
    pub liked: bool,
    pub replied: bool,
    pub reposted: bool,
    pub quoted: bool,
    pub clicked: bool,
    pub shared: bool,
    pub followed_author: bool,
    pub blocked: bool,
    pub muted: bool,
    pub reported: bool,
}

pub struct UserProfileStore {
    path: PathBuf,
    profiles: RwLock<HashMap<String, UserProfile>>,
}

impl UserProfileStore {
    pub async fn load(path: PathBuf) -> Result<Self, String> {
        let profiles = if path.exists() {
            let data = tokio::fs::read_to_string(&path)
                .await
                .map_err(|err| format!("failed to read profiles: {}", err))?;
            if data.trim().is_empty() {
                HashMap::new()
            } else {
                serde_json::from_str(&data)
                    .map_err(|err| format!("failed to parse profiles: {}", err))?
            }
        } else {
            HashMap::new()
        };

        Ok(Self {
            path,
            profiles: RwLock::new(profiles),
        })
    }

    pub async fn list(&self) -> Vec<UserProfile> {
        let guard = self.profiles.read().await;
        guard.values().cloned().collect()
    }

    pub async fn get(&self, user_id: &str) -> Option<UserProfile> {
        let guard = self.profiles.read().await;
        guard.get(user_id).cloned()
    }

    pub async fn upsert(&self, profile: UserProfile) -> Result<UserProfile, String> {
        let mut guard = self.profiles.write().await;
        guard.insert(profile.user_id.clone(), profile.clone());
        self.persist(&guard).await?;
        Ok(profile)
    }

    pub async fn update_history(
        &self,
        user_id: &str,
        history: Vec<EngagementEvent>,
    ) -> Result<UserProfile, String> {
        let mut guard = self.profiles.write().await;
        let profile = guard.get_mut(user_id).ok_or_else(|| {
            format!("user profile not found: {}", user_id)
        })?;
        profile.engagement_history = history;
        let updated = profile.clone();
        self.persist(&guard).await?;
        Ok(updated)
    }

    async fn persist(&self, profiles: &HashMap<String, UserProfile>) -> Result<(), String> {
        if let Some(parent) = self.path.parent() {
            ensure_dir(parent).await?;
        }
        let payload = serde_json::to_string_pretty(profiles)
            .map_err(|err| format!("failed to serialize profiles: {}", err))?;
        let tmp_path = self.path.with_extension("json.tmp");
        tokio::fs::write(&tmp_path, payload)
            .await
            .map_err(|err| format!("failed to write profiles: {}", err))?;
        tokio::fs::rename(&tmp_path, &self.path)
            .await
            .map_err(|err| format!("failed to finalize profiles: {}", err))?;
        Ok(())
    }
}

async fn ensure_dir(path: &Path) -> Result<(), String> {
    if path.exists() {
        return Ok(());
    }
    tokio::fs::create_dir_all(path)
        .await
        .map_err(|err| format!("failed to create profile dir: {}", err))
}
