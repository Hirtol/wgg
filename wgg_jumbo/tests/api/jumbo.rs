use crate::common::base_jumbo_api;
use crate::conditional_test;
use wgg_jumbo::clients::BaseApi;

#[tokio::test]
pub async fn test_promotion_tabs() {
    let api = conditional_test!(base_jumbo_api);

    let result = api.promotion_tabs().await.unwrap();

    assert!(result.tabs.iter().any(|tab| tab.id == "weekaanbiedingen"))
}

#[tokio::test]
pub async fn test_promotion_group() {
    let api = conditional_test!(base_jumbo_api);

    let current_promotion = api.promotion_tabs().await.unwrap();
    let latest = current_promotion
        .tabs
        .iter()
        .find(|tab| tab.id == "weekaanbiedingen")
        .unwrap();

    let result = api
        .promotion_group(&latest.id, &latest.runtimes[0].id, None, None)
        .await
        .unwrap();

    assert!(!result.categories.is_empty())
}
