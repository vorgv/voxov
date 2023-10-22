use vcli::client::Client;
use voxov::{config::Config, database::Database};

#[tokio::test]
async fn session_start() {
    let client = Client::zero().await;
    client.auth_session_start().await.unwrap();
}

#[tokio::test]
async fn session_refresh() {
    let mut client = Client::zero().await;
    client.update_session().await;
    let access = client.auth_session_refresh().await.unwrap();
    client.config.session.as_mut().unwrap().set_access(&access);
}

#[tokio::test]
async fn session_end() {
    let mut client = Client::zero().await;
    client.update_session().await;
    client.auth_session_end(false).await.unwrap();
    let access = client.auth_session_refresh().await.unwrap();
    client.config.session.as_mut().unwrap().set_access(&access);
    client.auth_session_end(true).await.unwrap();
}

#[tokio::test]
async fn session_sms() {
    let mut client = Client::zero().await;
    client.update_session().await;
    let (phone, message) = client.auth_sms_send_to().await.unwrap();
    let config = Config::new();
    let db = Database::new(&config, false).await;
    db.sms_sent("7357", &phone, &message).await.unwrap();
    client.auth_sms_sent(&phone, &message).await.unwrap();
}
