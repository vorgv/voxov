use vcli::client::Client;

#[tokio::test]
async fn ping() {
    let client = Client::zero().await;
    client.ping().await.unwrap();
}
