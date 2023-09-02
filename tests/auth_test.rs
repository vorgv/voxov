mod common;

use common::_post;
use std::str::FromStr;
use voxov::ir::id::Id;

#[tokio::test]
async fn session_start() {
    let response = _post()
        .await
        .header("type", "AuthSessionStart")
        .send()
        .await
        .unwrap();

    let headers = response.headers();
    assert!(Id::from_str(headers.get("access").unwrap().to_str().unwrap()).is_ok());
    assert!(Id::from_str(headers.get("refresh").unwrap().to_str().unwrap()).is_ok());
}

#[tokio::test]
async fn session_refresh() {
    // Invalid refresh token
    let response = _post()
        .await
        .header("type", "AuthSessionRefresh")
        .header("refresh", Id::zero().to_string())
        .send()
        .await
        .unwrap();

    let headers = response.headers();
    assert_eq!(headers.get("type").unwrap().to_str().unwrap(), "Error");

    // Refreshed access token
    let response = _post()
        .await
        .header("type", "AuthSessionStart")
        .send()
        .await
        .unwrap();

    let headers = response.headers();
    let access = headers.get("access").unwrap().to_str().unwrap();
    let refresh = headers.get("refresh").unwrap().to_str().unwrap();

    let response = _post()
        .await
        .header("type", "AuthSessionRefresh")
        .header("refresh", refresh)
        .send()
        .await
        .unwrap();

    assert_ne!(
        response.headers().get("access").unwrap().to_str().unwrap(),
        access
    );
}

#[tokio::test]
async fn session_end() {
    // End access
    let response = _post()
        .await
        .header("type", "AuthSessionStart")
        .send()
        .await
        .unwrap();

    let headers = response.headers();
    let access = headers.get("access").unwrap().to_str().unwrap();
    let refresh = headers.get("refresh").unwrap().to_str().unwrap();

    let response = _post()
        .await
        .header("type", "AuthSessionEnd")
        .header("access", access)
        .send()
        .await
        .unwrap();

    assert_eq!(
        response.headers().get("type").unwrap().to_str().unwrap(),
        "AuthSessionEnd"
    );

    let response = _post()
        .await
        .header("type", "AuthSessionEnd")
        .header("access", access)
        .send()
        .await
        .unwrap();

    assert_eq!(
        response.headers().get("type").unwrap().to_str().unwrap(),
        "Error"
    );

    let response = _post()
        .await
        .header("type", "AuthSessionRefresh")
        .header("refresh", refresh)
        .send()
        .await
        .unwrap();

    assert_eq!(
        response.headers().get("type").unwrap().to_str().unwrap(),
        "AuthSessionRefresh"
    );
    // End access and refresh

    let access = response.headers().get("access").unwrap().to_str().unwrap();

    let response = _post()
        .await
        .header("type", "AuthSessionEnd")
        .header("access", access)
        .header("refresh", refresh)
        .send()
        .await
        .unwrap();

    assert_eq!(
        response.headers().get("type").unwrap().to_str().unwrap(),
        "AuthSessionEnd"
    );

    let response = _post()
        .await
        .header("type", "AuthSessionRefresh")
        .header("refresh", refresh)
        .send()
        .await
        .unwrap();

    assert_eq!(
        response.headers().get("type").unwrap().to_str().unwrap(),
        "Error"
    );
}

#[tokio::test]
async fn session_sms() {}
