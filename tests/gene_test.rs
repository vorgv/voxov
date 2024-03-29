mod common;
use common::new_user;

#[tokio::test]
async fn gene_meta() {
    let (client, _) = new_user().await;
    client.gene_meta(None, "info_1").await.unwrap();
}

#[tokio::test]
async fn gene_call() {
    let (client, _) = new_user().await;
    client.gene_call(None, "info_1", None).await.unwrap();
}
