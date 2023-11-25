use voxov::{
    database::{namespace::UID2CREDIT, ns, Database},
    ir::Id,
};

mod common;
use common::new_user;

#[tokio::test]
async fn cost_pay() {
    let (client, _) = new_user().await;
    client.cost_pay().await.unwrap();
}

#[tokio::test]
async fn cost_get() {
    let (client, uid) = new_user().await;
    let credit: i64 = client.cost_get().await.unwrap().parse().unwrap();
    let credit_in_db = get_credit(&uid).await;
    assert_eq!(credit, credit_in_db);
}

#[tokio::test]
async fn cost_check_in() {
    let (client, uid) = new_user().await;
    let credit_before = get_credit(&uid).await;
    let award: i64 = client.cost_check_in().await.unwrap().parse().unwrap();
    let credit_after = get_credit(&uid).await;
    assert_eq!(credit_before + award, credit_after);
}

async fn get_credit(uid: &String) -> i64 {
    let db = Database::default().await;
    db.get::<&[u8], i64>(&ns(UID2CREDIT, &Id::try_from(uid).unwrap())[..])
        .await
        .unwrap()
}
