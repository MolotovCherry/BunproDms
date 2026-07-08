use std::fmt::{Debug, Display};

use bon::Builder;

use crate::config::config_builder::{IsUnset, SetApiToken, State};

#[derive(Clone, Debug, Builder)]
pub struct Config {
    #[builder(into)]
    pub user_agent: Option<String>,
    #[builder(setters(vis = "", name = api_token_internal))]
    pub api_token: Token,
    /// Allow to use Settings->API token instead of cookie token
    ///
    /// WARNING: Can be dangerous as API token isn't rotated
    #[builder(default)]
    pub dangerously_authenticate_using_api_token: bool,
}

impl<S: State> ConfigBuilder<S> {
    pub fn api_token<P: Into<String>>(self, value: P) -> ConfigBuilder<SetApiToken<S>>
    where
        S::ApiToken: IsUnset,
    {
        self.api_token_internal(Token(value.into()))
    }
}

#[derive(Clone)]
pub struct Token(String);

impl Token {
    pub fn new(token: &str) -> Self {
        Self(token.to_owned())
    }

    pub fn secret(&self) -> &str {
        &self.0
    }
}

impl Debug for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_tuple("Token").field(&"****").finish()
    }
}

impl Display for Token {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "****")
    }
}
