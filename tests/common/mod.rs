pub fn url() -> String {
    format!(
        "http://{}",
        voxov::config::Config::new().http_addr.to_string()
    )
}

pub async fn _post() -> reqwest::RequestBuilder {
    reqwest::Client::new().post(url())
}
