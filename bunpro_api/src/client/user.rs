mod due;

use crate::client::BunproApi;
use due::UserDueApi;

#[derive(Copy, Clone, Debug)]
pub struct UserApi<'a> {
    client: &'a BunproApi,
}

impl<'a> UserApi<'a> {
    pub fn new(client: &'a BunproApi) -> Self {
        Self { client }
    }

    pub fn due(&self) -> UserDueApi<'_> {
        UserDueApi::new(self.client)
    }
}
