use super::{get_header, Client, Result};
use crate::handle_error;

impl Client {
    /// Get the link to pay.
    pub async fn cost_pay(&self) -> Result<String> {
        let response = self
            .post()
            .header("type", "CostPay")
            .header("access", &self.config.session.as_ref().unwrap().access)
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
            .header("access", &self.config.session.as_ref().unwrap().access)
            .send()
            .await?;
        handle_error!(response);
        let credit = get_header(&response, "credit");
        Ok(credit)
    }
}
