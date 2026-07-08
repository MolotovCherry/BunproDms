mod user;
mod user_stats;

use reqwest::{Client, ClientBuilder};
use user::UserApi;
use user_stats::UserStatsApi;

use crate::{config::Config, request::ApiRequest};

#[derive(Debug)]
pub struct BunproApi {
    pub config: Config,
    pub(crate) http: Client,
}

impl BunproApi {
    pub fn new(config: Config) -> reqwest::Result<Self> {
        let mut builder = ClientBuilder::new();

        builder = match &config.user_agent {
            Some(ua) => builder.user_agent(ua),
            None => builder.user_agent(concat!("BunproClient/", env!("CARGO_PKG_VERSION"))),
        };

        let this = Self {
            http: builder.build()?,
            config,
        };

        Ok(this)
    }

    pub fn user(&self) -> UserApi<'_> {
        UserApi::new(self)
    }

    pub fn user_stats(&self) -> UserStatsApi<'_> {
        UserStatsApi::new(self)
    }
}

impl BunproApi {
    pub(crate) fn req(&self) -> ApiRequest<'_> {
        ApiRequest::new(self)
    }
}
