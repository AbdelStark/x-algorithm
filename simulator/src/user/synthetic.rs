use rand::{rngs::StdRng, Rng, SeedableRng};
use std::time::{SystemTime, UNIX_EPOCH};

use crate::user::profile::{ActionFlags, EngagementEvent, UserProfile};

const DEFAULT_HISTORY_LEN: usize = 50;

pub fn generate_synthetic_history(profile: &UserProfile, seed: u64) -> Vec<EngagementEvent> {
    let mut rng = StdRng::seed_from_u64(seed);
    let engagement_rate = estimate_engagement_rate(profile);

    let mut history = Vec::new();
    let now = current_timestamp();

    for idx in 0..DEFAULT_HISTORY_LEN {
        if rng.gen::<f64>() >= engagement_rate {
            continue;
        }

        let actions = sample_actions(&mut rng);
        history.push(EngagementEvent {
            post_id: format!("synthetic_{}", idx),
            author_id: format!("author_{}", rng.gen_range(0..200)),
            timestamp: now - (idx as i64 * 3600),
            actions,
        });
    }

    history
}

fn estimate_engagement_rate(profile: &UserProfile) -> f64 {
    let follower_factor = (profile.followers.max(1) as f64).log10().max(1.0) / 10.0;
    let age_factor = (profile.account_age_days as f64 / 365.0).min(5.0) / 10.0;
    let verified_bonus = if profile.verified { 0.02 } else { 0.0 };
    let base = 0.04 + follower_factor + age_factor + verified_bonus;
    base.clamp(0.02, 0.18)
}

fn sample_actions(rng: &mut StdRng) -> ActionFlags {
    let liked = rng.gen::<f64>() < 0.75;
    let replied = rng.gen::<f64>() < 0.12;
    let reposted = rng.gen::<f64>() < 0.08;
    let quoted = rng.gen::<f64>() < 0.05;
    let clicked = rng.gen::<f64>() < 0.2;
    let shared = rng.gen::<f64>() < 0.06;
    let followed_author = rng.gen::<f64>() < 0.02;
    let blocked = rng.gen::<f64>() < 0.005;
    let muted = rng.gen::<f64>() < 0.01;
    let reported = rng.gen::<f64>() < 0.003;

    ActionFlags {
        liked,
        replied,
        reposted,
        quoted,
        clicked,
        shared,
        followed_author,
        blocked,
        muted,
        reported,
    }
}

fn current_timestamp() -> i64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|duration| duration.as_secs() as i64)
        .unwrap_or(0)
}
