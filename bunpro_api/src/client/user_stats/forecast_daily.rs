use crate::{
    client::BunproApi, objects::ForecastDaily, request::ApiResult,
    urls::USER_STATS_FORECAST_DAILY,
};

pub struct UserStatsForecastDailyApi<'a> {
    client: &'a BunproApi,
}

impl<'a> UserStatsForecastDailyApi<'a> {
    pub fn new(client: &'a BunproApi) -> Self {
        Self { client }
    }

    pub async fn get(self) -> ApiResult<ForecastDaily> {
        self.client.req().get(USER_STATS_FORECAST_DAILY).await
    }
}
