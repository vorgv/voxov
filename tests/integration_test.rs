mod layer;

#[tokio::test]
async fn test_voxov() {
    tokio::spawn(voxov::run());
    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
    layer::all().await;
}
