pub mod objects;

pub use bunpro_api;

use bunpro_api::{
    client::BunproApi,
    config::Config,
    objects::TotalDue,
    request::{ApiResult, RequestError},
    reqwest,
};
use snafu::{ResultExt, Snafu};

use crate::objects::{Forecast, ForecastEntry, ForecastObject};

#[derive(Debug, Snafu)]
pub enum BunproClientError {
    #[snafu(display("{source}"))]
    RequestError { source: RequestError },
}

pub struct BunproClient {
    client: BunproApi,
    pub total_due: TotalDue,
    pub forecast: Forecast,
}

impl BunproClient {
    pub fn builder() -> BunproClientBuilder {
        BunproClientBuilder::new()
    }

    pub fn config_mut(&mut self, f: impl FnOnce(&mut Config)) {
        f(&mut self.client.config);
    }

    /// Refreshes total_due and forecast
    pub async fn refresh_forecast(&mut self) -> ApiResult<()> {
        let user = self.client.user();
        let user_stats = self.client.user_stats();

        let due = user.due().get();
        let forecast_daily = user_stats.forecast_daily().get();
        let forecast_hourly = user_stats.forecast_hourly().get();

        let (total_due, forecast_daily, forecast_hourly) =
            tokio::try_join!(due, forecast_daily, forecast_hourly)?;

        self.total_due = total_due;

        let a = (total_due.total_due_grammar, forecast_daily.grammar);
        let b = (total_due.total_due_vocab, forecast_daily.vocab);
        let c = (total_due.total_due_grammar, forecast_hourly.grammar);
        let d = (total_due.total_due_vocab, forecast_hourly.vocab);

        let daily_grammar = ForecastObject::from(a);
        let daily_vocab = ForecastObject::from(b);
        let hourly_grammar = ForecastObject::from(c);
        let hourly_vocab = ForecastObject::from(d);

        let daily_entry = ForecastEntry {
            grammar: daily_grammar,
            vocab: daily_vocab,
        };

        let hourly_entry = ForecastEntry {
            grammar: hourly_grammar,
            vocab: hourly_vocab,
        };

        self.forecast = Forecast {
            daily: daily_entry,
            hourly: hourly_entry,
        };

        Ok(())
    }
}

#[derive(Debug, Snafu)]
pub enum BunproClientBuilderError {
    #[snafu(display("{source}"))]
    Reqwest { source: reqwest::Error },
    #[snafu(display("Api token is required"))]
    MissingApiToken,
}

pub struct BunproClientBuilder {
    api_token: Option<String>,
    dangerously_authenticate_using_api_token: bool,
}

impl BunproClientBuilder {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self {
            api_token: None,
            dangerously_authenticate_using_api_token: false,
        }
    }

    pub fn api_token(mut self, token: &str) -> Self {
        self.api_token = Some(token.to_owned());
        self
    }

    pub fn dangerously_authenticate_using_api_token(mut self, state: bool) -> Self {
        self.dangerously_authenticate_using_api_token = state;
        self
    }

    pub fn build(self) -> Result<BunproClient, BunproClientBuilderError> {
        let token = self
            .api_token
            .ok_or(BunproClientBuilderError::MissingApiToken)?;

        let config = Config::builder()
            .api_token(token)
            .dangerously_authenticate_using_api_token(self.dangerously_authenticate_using_api_token)
            .build();

        let client = BunproApi::new(config).context(ReqwestSnafu)?;

        let this = BunproClient {
            client,
            total_due: Default::default(),
            forecast: Default::default(),
        };

        Ok(this)
    }
}
