use rand::{distributions::Alphanumeric, Rng};
use vcli::{client::Client, config::Session};
use voxov::database::Database;

/// Authenticate user with number, return (client, uid).
pub async fn new_user() -> (Client, String) {
    let mut client = Client::zero().await;
    let (access, refresh) = client.auth_session_start().await.unwrap();
    client.config.session = Some(Session::new(&access, &refresh));
    let (phone, message) = client.auth_sms_send_to().await.unwrap();
    let db = Database::default().await;
    let number: String = rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect();
    db.sms_sent(&number, &phone, &message).await.unwrap();
    let uid = client.auth_sms_sent(&phone, &message).await.unwrap();
    (client, uid)
}
