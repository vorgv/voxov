use rand::{Rng, distr::Alphanumeric};
use std::str::FromStr;
use vcli::{client::Client, config::Session};
use voxov::database::Database;

/// Authenticate user with number, return (client, uid).
pub async fn new_user() -> (Client, String) {
    let mut client = Client::zero().await;
    let (access, refresh) = client.auth_session_start().await.unwrap();
    client.config.session = Some(Session::new(&access, &refresh));
    let (phone, message) = client.auth_sms_send_to().await.unwrap();
    let db = Database::default().await;
    let number = random_string(16);
    // Simulate carrier callback: record that user's phone sent the SMS
    let message_id = voxov::ir::Id::from_str(&message).unwrap();
    db.sms_sent(&number, &phone, &message_id.0).await.unwrap();
    let uid = client.auth_sms_sent(&phone, &message).await.unwrap();
    (client, uid)
}

/// Generate a random String with length n.
pub fn random_string(n: usize) -> String {
    rand::rng()
        .sample_iter(Alphanumeric)
        .take(n)
        .map(char::from)
        .collect()
}
