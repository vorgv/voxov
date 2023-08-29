use reqwest;
use voxov::config::Config;

pub async fn ping() {
    let http_addr = Config::new().http_addr.to_string();
    let body = reqwest::get(format!("http://{}", http_addr))
        .await
        .expect("get")
        .text()
        .await
        .expect("text");
    assert_eq!("PONG", body);
}
