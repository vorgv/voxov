use std::str::FromStr;
use vcli::{client::Client, config::Session};
use voxov::database::Database;
use voxov::ir::Id;

mod common;

async fn token_exists(db: &Database, token: &[u8], is_access: bool) -> bool {
    if is_access {
        db.get_access(token).await.unwrap().is_some()
    } else {
        // For refresh, we can't check without extending TTL, so query directly
        let result = db
            .scylla
            .execute_unpaged(&db.stmts.select_session, (token,))
            .await
            .unwrap();
        if let Some(row) = result
            .into_rows_result()
            .unwrap()
            .rows::<(Vec<u8>, i8)>()
            .unwrap()
            .next()
        {
            let (_, kind) = row.unwrap();
            return kind == 1; // refresh token
        }
        false
    }
}

async fn tokens_exist(access: &str, refresh: &str) -> (bool, bool) {
    let db = Database::default().await;
    let access_bytes = Id::from_str(access).unwrap().0;
    let refresh_bytes = Id::from_str(refresh).unwrap().0;

    (
        token_exists(&db, &access_bytes, true).await,
        token_exists(&db, &refresh_bytes, false).await,
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
    let access_bytes = Id::from_str(&session.access).unwrap().0;
    let refresh_bytes = Id::from_str(&session.refresh).unwrap().0;

    let access_uid = db.get_access(&access_bytes).await.unwrap().unwrap();

    // Query refresh token directly from ScyllaDB
    let result = db
        .scylla
        .execute_unpaged(&db.stmts.select_session, (&refresh_bytes[..],))
        .await
        .unwrap();
    let (uid_bytes, _): (Vec<u8>, i8) = result
        .into_rows_result()
        .unwrap()
        .rows::<(Vec<u8>, i8)>()
        .unwrap()
        .next()
        .unwrap()
        .unwrap();
    let mut refresh_uid = Id::zero();
    refresh_uid.0.copy_from_slice(&uid_bytes);

    (access_uid.to_string(), refresh_uid.to_string())
}
