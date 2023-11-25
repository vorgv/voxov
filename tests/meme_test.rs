use blake3;
use serde_json::Value;
use std::collections::HashMap;

mod common;
use common::{new_user, random_string};

const DAYS: u32 = 1;
const SIZE: usize = 1000;

#[tokio::test]
async fn meme_put() {
    let (client, _) = new_user().await;
    let raw = random_string(SIZE);
    let hash = client.meme_put(DAYS, raw.clone().into()).await.unwrap();
    assert_eq!(blake3::hash(raw.as_bytes()).to_hex().as_str(), hash);
}

#[tokio::test]
async fn meme_get() {
    let (client, _) = new_user().await;
    let raw = random_string(SIZE);
    let hash = client.meme_put(DAYS, raw.clone().into()).await.unwrap();
    let got = client.meme_get(false, hash).await.unwrap();
    assert_eq!(raw, got);
}

#[tokio::test]
async fn meme_meta() {
    let (client, uid) = new_user().await;
    let raw = random_string(SIZE);
    let hash = client.meme_put(1, raw.clone().into()).await.unwrap();
    let meta = client.meme_meta(hash.clone()).await.unwrap();
    println!("{}", meta);
    let hm: HashMap<String, Value> = serde_json::from_str(&meta).unwrap();
    assert_eq!(hm.get("uid").unwrap(), &uid);
    assert_eq!(hm.get("hash").unwrap(), &hash);
    assert_eq!(hm.get("size").unwrap(), SIZE);
    assert_eq!(hm.get("pub").unwrap(), false);
    assert_eq!(hm.get("tip").unwrap(), 0);
}
