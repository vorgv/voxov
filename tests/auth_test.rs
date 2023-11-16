use std::str::FromStr;
use vcli::{client::Client, config::Session};
use voxov::database::{
    namespace::{ACCESS, REFRESH},
    ns, Database,
};
use voxov::ir::Id;

mod common;

async fn token_exists(db: &Database, k: u8, id: &str) -> bool {
    db.exists(&ns(k, &Id::from_str(id).unwrap())[..])
        .await
        .unwrap()
        > 0
}

async fn tokens_exist(access: &str, refresh: &str) -> (bool, bool) {
    let db = Database::default().await;

    (
        token_exists(&db, ACCESS, access).await,
        token_exists(&db, REFRESH, refresh).await,
    )
}

async fn session_tokens_exist(session: &Session) -> (bool, bool) {
    tokens_exist(&session.access, &session.refresh).await
}

#[tokio::test]
async fn session_start() {
    let client = Client::zero().await;
    let (access, refresh) = client.auth_session_start().await.unwrap();
    assert_eq!(tokens_exist(&access, &refresh).await, (true, true));
}

#[tokio::test]
async fn session_refresh() {
    let mut client = Client::zero().await;
    let (access, refresh) = client.auth_session_start().await.unwrap();
    client.config.session = Some(Session::new(&access, &refresh));

    client.auth_session_end(false).await.unwrap();

    let session = client.config.session.clone().unwrap();
    assert_eq!(session_tokens_exist(&session).await, (false, true));

    let access = client.auth_session_refresh().await.unwrap();
    client.config.session = Some(Session::new(&access, &refresh));

    let session = client.config.session.clone().unwrap();
    assert_eq!(session_tokens_exist(&session).await, (true, true));
}

#[tokio::test]
async fn session_end() {
    let mut client = Client::zero().await;
    let (access, refresh) = client.auth_session_start().await.unwrap();
    client.config.session = Some(Session::new(&access, &refresh));

    // client.auth_session_end(false) is covered by test session_refresh.

    client.auth_session_end(true).await.unwrap();
    let session = &client.config.session.unwrap();
    assert_eq!(session_tokens_exist(session).await, (false, false));
}

#[tokio::test]
async fn session_sms() {
    let (client, uid) = common::new_user().await;
    let (access_uid, refresh_uid) = get_tokens(client).await;
    if uid != access_uid || uid != refresh_uid {
        panic!()
    }
}

async fn get_tokens(client: Client) -> (String, String) {
    let db = Database::default().await;
    let session = &client.config.session.unwrap();
    let access_uid: Vec<u8> = db
        .get(&ns(ACCESS, &Id::from_str(&session.access).unwrap())[..])
        .await
        .unwrap();
    let refresh_uid: Vec<u8> = db
        .get(&ns(REFRESH, &Id::from_str(&session.refresh).unwrap())[..])
        .await
        .unwrap();
    (
        Id::try_from(access_uid).unwrap().to_string(),
        Id::try_from(refresh_uid).unwrap().to_string(),
    )
}
