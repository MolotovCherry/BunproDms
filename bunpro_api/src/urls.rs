use const_format::formatcp;

pub const BASE: &str = "https://api.bunpro.jp/api/frontend";

pub const USER: &str = formatcp!("{BASE}/user");
pub const USER_DUE: &str = formatcp!("{USER}/due");

pub const USER_STATS: &str = formatcp!("{BASE}/user_stats");
pub const USER_STATS_FORECAST_DAILY: &str = formatcp!("{USER_STATS}/forecast_daily");
pub const USER_STATS_FORECAST_HOURLY: &str = formatcp!("{USER_STATS}/forecast_hourly");
