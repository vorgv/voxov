mod api;

pub async fn all() {
    api::ping().await;
}
