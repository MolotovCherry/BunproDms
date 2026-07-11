use std::fmt::{self, Display};

use log::{error, trace};
use reqwest::{IntoUrl, StatusCode};
use serde::de::DeserializeOwned;
use serde::{Deserialize, Serialize};
use snafu::{ResultExt, Snafu};

use crate::client::BunproApi;

pub type ApiResult<T> = Result<T, RequestError>;

#[derive(Copy, Clone, Debug)]
pub(crate) enum RequestMethod {
    Get,
    Patch,
    Post,
    Put,
    Delete,
}

impl Display for RequestMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            RequestMethod::Get => write!(f, "GET"),
            RequestMethod::Patch => write!(f, "PATCH"),
            RequestMethod::Post => write!(f, "POST"),
            RequestMethod::Put => write!(f, "PUT"),
            RequestMethod::Delete => write!(f, "DELETE"),
        }
    }
}

#[derive(Snafu, Debug)]
pub enum RequestError {
    #[snafu(display("Error occurred during request: {source}"))]
    ReqwestError { source: reqwest::Error },
    #[snafu(display("Invalid token (Expired access tokens, Invalid access tokens, etc.)"))]
    Unauthorized { codes: Option<Vec<String>> },
    #[snafu(display("Invalid Parameters"))]
    InvalidParameters { codes: Option<Vec<String>> },
    #[snafu(display("Access is forbidden (DoS detected etc.)"))]
    Forbidden { codes: Option<Vec<String>> },
    #[snafu(display("Not found"))]
    NotFound,
    #[snafu(display("[{code}]: {codes:?}"))]
    StatusCode {
        code: StatusCode,
        codes: Option<Vec<String>>,
    },
    #[snafu(display("{source}"))]
    ParseError { source: serde_json::Error },
}

#[derive(Deserialize)]
struct ApiError {
    errors: Vec<ApiErrorCode>,
}

#[derive(Deserialize)]
struct ApiErrorCode {
    code: String,
}

#[derive(Clone, Debug)]
pub(crate) struct ApiRequest<'a> {
    client: &'a BunproApi,
}

impl<'a> ApiRequest<'a> {
    pub(crate) fn new(client: &'a BunproApi) -> Self {
        Self { client }
    }

    pub(crate) async fn get<D>(&self, url: impl IntoUrl) -> ApiResult<D>
    where
        D: DeserializeOwned,
    {
        self.api_request(url, RequestMethod::Get, None::<()>).await
    }

    #[expect(unused)]
    pub(crate) async fn delete<D>(&self, url: impl IntoUrl) -> ApiResult<D>
    where
        D: DeserializeOwned,
    {
        self.api_request(url, RequestMethod::Delete, None::<()>)
            .await
    }

    #[expect(unused)]
    pub(crate) async fn patch<D, P: Serialize>(
        &self,
        url: impl IntoUrl,
        data: Option<P>,
    ) -> ApiResult<D>
    where
        D: DeserializeOwned,
    {
        self.api_request(url, RequestMethod::Patch, data).await
    }

    #[expect(unused)]
    pub(crate) async fn post<D, P: Serialize>(
        &self,
        url: impl IntoUrl,
        data: Option<P>,
    ) -> ApiResult<D>
    where
        D: DeserializeOwned,
    {
        self.api_request(url, RequestMethod::Post, data).await
    }

    #[expect(unused)]
    pub(crate) async fn put<D, P: Serialize>(
        &self,
        url: impl IntoUrl,
        data: Option<P>,
    ) -> ApiResult<D>
    where
        D: DeserializeOwned,
    {
        self.api_request(url, RequestMethod::Put, data).await
    }

    async fn api_request<D, P: Serialize>(
        &self,
        url: impl IntoUrl,
        method: RequestMethod,
        data: Option<P>,
    ) -> ApiResult<D>
    where
        D: DeserializeOwned,
    {
        let mut url = url.into_url().context(ReqwestSnafu)?;

        let use_settings_token = match self.client.config.dangerously_authenticate_using_api_token {
            true => "true",
            false => "false",
        };

        url.query_pairs_mut().append_pair(
            "dangerously_authenticate_using_api_token",
            use_settings_token,
        );

        trace!("{method} {url}");

        let mut request = match method {
            RequestMethod::Get => self.client.http.get(url),
            RequestMethod::Delete => self.client.http.delete(url),
            RequestMethod::Put => self.client.http.put(url),
            RequestMethod::Patch => self.client.http.patch(url),
            RequestMethod::Post => self.client.http.post(url),
        };

        if let Some(data) = &data {
            request = request.form(data);
        }

        let request = request.bearer_auth(self.client.config.api_token.secret());

        let response = request.send().await.context(ReqwestSnafu)?;

        let status = response.status();

        if status.is_client_error() {
            let text = response.text().await.unwrap_or_default();

            let codes = serde_json::from_str::<ApiError>(&text)
                .map(|e| e.errors.into_iter().map(|i| i.code).collect())
                .ok();

            let err = match status {
                StatusCode::BAD_REQUEST => RequestError::InvalidParameters { codes },
                StatusCode::UNAUTHORIZED => RequestError::Unauthorized { codes },
                StatusCode::FORBIDDEN => RequestError::Forbidden { codes },
                StatusCode::NOT_FOUND => RequestError::NotFound,
                code => RequestError::StatusCode { code, codes },
            };

            error!(status:%, body:% = text, error:% = err; "api error");

            return Err(err);
        }

        let text = response.text().await.context(ReqwestSnafu)?;

        let data = match serde_json::from_str(&text).context(ParseSnafu) {
            Ok(s) => {
                trace!(body:% = text; "api response body");
                s
            }

            Err(e) => {
                error!(body:% = text, error:% = e; "failed to parse api reponse");
                return Err(e);
            }
        };

        Ok(data)
    }
}
