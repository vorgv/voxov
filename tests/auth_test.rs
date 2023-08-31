mod common;

use common::_post;

#[tokio::test]
async fn session_start() {
    let response = _post()
        .await
        .header("type", "AuthSessionStart")
        .send()
        .await
        .unwrap();

    let headers = response.headers();
    assert!(headers.contains_key("access"));
    assert!(headers.contains_key("refresh"));

    let id_len = voxov::ir::id::IDL * 2;
    assert_eq!(headers.get("access").unwrap().len(), id_len);
    assert_eq!(headers.get("refresh").unwrap().len(), id_len);
}

#[tokio::test]
async fn session_refresh() {}

#[tokio::test]
async fn session_end() {}

#[tokio::test]
async fn session_sms() {}
