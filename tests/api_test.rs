mod common;

#[tokio::test]
async fn ping() {
    let url = common::url();
    let response = reqwest::get(&url).await;

    if response.is_err() {
        panic!("Is VOxOV running at {}?", url);
    }

    let body = response.unwrap().text().await;

    if body.is_err() {
        panic!("Is it a VOxOV instance at {}?", url);
    }

    assert_eq!(body.unwrap(), "PONG", "Is it a VOxOV instance at {}?", url);
}
