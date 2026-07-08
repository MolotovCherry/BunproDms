use crate::{
    client::BunproApi, objects::ForecastHourly, request::ApiResult,
    urls::USER_STATS_FORECAST_HOURLY,
};

pub struct UserStatsForecastHourlyApi<'a> {
    client: &'a BunproApi,
}

impl<'a> UserStatsForecastHourlyApi<'a> {
    pub fn new(client: &'a BunproApi) -> Self {
        Self { client }
    }

    pub async fn get(self) -> ApiResult<ForecastHourly> {
        self.client.req().get(USER_STATS_FORECAST_HOURLY).await
    }
}
