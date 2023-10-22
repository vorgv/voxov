use vcli::client::Client;

#[tokio::test]
async fn session_start() {
    let client = Client::default().await;
    client.auth_session_start().await.unwrap();
}

#[tokio::test]
async fn session_refresh() {
    let client = Client::default().await;
    client.auth_session_refresh().await.unwrap();
}

#[tokio::test]
async fn session_end() {
    let client = Client::default().await;
    client.auth_session_end(false).await.unwrap();
    client.auth_session_refresh().await.unwrap();
    client.auth_session_end(true).await.unwrap();
}

#[tokio::test]
async fn session_sms() {}
