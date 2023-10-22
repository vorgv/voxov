use vcli::client::Client;

#[tokio::test]
async fn ping() {
    let client = Client::default().await;
    client.ping().await.unwrap();
}
