use super::{get_header, Client, Result};
use crate::handle_error;

impl Client {
    /// Get the link to pay.
    pub async fn cost_pay(&self) -> Result<String> {
        let response = self
            .post()
            .header("type", "CostPay")
            .header("access", &self.get_access()?)
            .header("vendor", "00000000000000000000000000000000")
            .send()
            .await?;
        handle_error!(response);
        let uri = get_header(&response, "uri");
        Ok(uri)
    }

    /// Get user balance.
    pub async fn cost_get(&self) -> Result<String> {
        let response = self
            .post()
            .header("type", "CostGet")
            .header("access", &self.get_access()?)
            .send()
            .await?;
        handle_error!(response);
        let credit = get_header(&response, "credit");
        Ok(credit)
    }

    /// Check in.
    pub async fn cost_check_in(&self) -> Result<String> {
        let response = self
            .post()
            .header("type", "CostCheckIn")
            .header("access", &self.get_access()?)
            .send()
            .await?;
        handle_error!(response);
        let award = get_header(&response, "award");
        Ok(award)
    }
}
