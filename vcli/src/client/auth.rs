use super::{get_header, Client, Result};
use crate::handle_error;

impl Client {
    /// Get access and refresh tokens.
    pub async fn auth_session_start(&self) -> Result<(String, String)> {
        let response = self
            .post()
            .header("type", "AuthSessionStart")
            .send()
            .await?;
        handle_error!(response);
        let access = get_header(&response, "access");
        let refresh = get_header(&response, "refresh");
        Ok((access, refresh))
    }

    /// Get a new access with refresh token.
    pub async fn auth_session_refresh(&self) -> Result<String> {
        let response = self
            .post()
            .header("type", "AuthSessionRefresh")
            .header("refresh", &self.get_refresh()?)
            .send()
            .await?;
        handle_error!(response);
        let access = get_header(&response, "access");
        Ok(access)
    }

    /// Deactivate tokens.
    pub async fn auth_session_end(&self, drop_refresh: bool) -> Result<()> {
        let mut builder = self
            .post()
            .header("type", "AuthSessionEnd")
            .header("access", &self.get_access()?);
        if drop_refresh {
            builder = builder.header("refresh", &self.config.session.as_ref().unwrap().refresh);
        }
        let response = builder.send().await?;
        handle_error!(response);
        Ok(())
    }

    /// Get where to send SMS.
    pub async fn auth_sms_send_to(&self) -> Result<(String, String)> {
        let response = self
            .post()
            .header("type", "AuthSmsSendTo")
            .header("access", &self.get_access()?)
            .header("refresh", &self.get_refresh()?)
            .send()
            .await?;
        handle_error!(response);
        let phone = get_header(&response, "phone");
        let message = get_header(&response, "message");
        Ok((phone, message))
    }

    /// Notify the server that SMS is sent.
    pub async fn auth_sms_sent(&self, phone: &str, message: &str) -> Result<String> {
        let response = self
            .post()
            .header("type", "AuthSmsSent")
            .header("access", &self.get_access()?)
            .header("refresh", &self.get_refresh()?)
            .header("phone", phone)
            .header("message", message)
            .send()
            .await?;
        handle_error!(response);
        let uid = get_header(&response, "uid");
        Ok(uid)
    }
}
