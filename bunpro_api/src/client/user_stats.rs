mod forecast_daily;
mod forecast_hourly;

use crate::client::BunproApi;
use forecast_daily::UserStatsForecastDailyApi;
use forecast_hourly::UserStatsForecastHourlyApi;

#[derive(Copy, Clone, Debug)]
pub struct UserStatsApi<'a> {
    client: &'a BunproApi,
}

impl<'a> UserStatsApi<'a> {
    pub fn new(client: &'a BunproApi) -> Self {
        Self { client }
    }

    pub fn forecast_daily(&self) -> UserStatsForecastDailyApi<'_> {
        UserStatsForecastDailyApi::new(self.client)
    }

    pub fn forecast_hourly(&self) -> UserStatsForecastHourlyApi<'_> {
        UserStatsForecastHourlyApi::new(self.client)
    }
}
