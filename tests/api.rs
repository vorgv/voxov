use reqwest;
use voxov::config::Config;

#[tokio::test]
pub async fn ping() {
    let http_addr = Config::new().http_addr.to_string();
    let response = reqwest::get(format!("http://{}", http_addr)).await;

    if response.is_err() {
        panic!("Is VOxOV running at {}?", http_addr);
    }

    let body = response.unwrap().text().await;

    if body.is_err() {
        panic!("Is it a VOxOV instance at {}?", http_addr);
    }

    assert_eq!(
        body.unwrap(),
        "PONG",
        "Is it a VOxOV instance at {}?",
        http_addr
    );
}
