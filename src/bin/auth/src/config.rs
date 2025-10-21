use chrono::Duration;

pub const JWT_LIFETIME: Duration = Duration::hours(1);
pub const KEY_PAIR_LIFETIME: Duration = Duration::hours(512);

pub const MIN_USERNAME_LENGHT: usize = 5;
pub const MAX_USERNAME_LENGHT: usize = 20;

pub const MIN_PASSWORD_LENGHT: usize = 5;
pub const MAX_PASSWORD_LENGHT: usize = 64;
