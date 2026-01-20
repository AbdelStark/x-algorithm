pub mod profile;
pub mod synthetic;

use crate::SimulatorInput;

pub use profile::{ActionFlags, EngagementEvent, UserProfile, UserProfileStore};
pub use synthetic::generate_synthetic_history;

pub fn action_flags_to_vector(flags: &ActionFlags) -> Vec<f32> {
    let mut values = vec![0.0; 19];
    values[0] = bool_to_value(flags.liked);
    values[1] = bool_to_value(flags.replied);
    values[2] = bool_to_value(flags.reposted);
    values[4] = bool_to_value(flags.clicked);
    values[7] = bool_to_value(flags.shared);
    values[11] = bool_to_value(flags.quoted);
    values[12] = bool_to_value(flags.quoted && flags.clicked);
    values[13] = bool_to_value(flags.followed_author);
    values[15] = bool_to_value(flags.blocked);
    values[16] = bool_to_value(flags.muted);
    values[17] = bool_to_value(flags.reported);
    values
}

pub fn profile_from_input(user_id: String, input: &SimulatorInput) -> UserProfile {
    UserProfile {
        user_id,
        followers: input.followers,
        following: input.following,
        account_age_days: input.account_age_days,
        verified: input.verified,
        engagement_history: Vec::new(),
    }
}

fn bool_to_value(value: bool) -> f32 {
    if value {
        1.0
    } else {
        0.0
    }
}
