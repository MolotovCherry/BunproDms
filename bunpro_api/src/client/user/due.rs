use crate::{client::BunproApi, objects::TotalDue, request::ApiResult, urls::USER_DUE};

pub struct UserDueApi<'a> {
    client: &'a BunproApi,
}

impl<'a> UserDueApi<'a> {
    pub fn new(client: &'a BunproApi) -> Self {
        Self { client }
    }

    pub async fn get(self) -> ApiResult<TotalDue> {
        self.client.req().get(USER_DUE).await
    }
}
