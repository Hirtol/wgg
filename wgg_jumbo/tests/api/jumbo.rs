use crate::common::base_jumbo_api;
use crate::conditional_test;
use wgg_jumbo::BaseApi;

#[tokio::test]
pub async fn test_promotion_tabs() {
    let api = conditional_test!(base_jumbo_api);

    let result = api.promotion_tabs().await.unwrap();

    assert!(result.tabs.iter().any(|tab| tab.id == "actieprijs"))
}

#[tokio::test]
pub async fn test_promotion_group() {
    let api = conditional_test!(base_jumbo_api);

    let current_promotion = api.promotion_tabs().await.unwrap();
    let latest = current_promotion
        .tabs
        .iter()
        .find(|tab| tab.id == "actieprijs")
        .unwrap();

    let result = api
        .promotion_content(&latest.id, &latest.runtimes[0].id, None, None)
        .await
        .unwrap();

    assert!(!result.groups.is_empty())
}

#[tokio::test]
pub async fn test_autocomplete() {
    let api = conditional_test!(base_jumbo_api);

    let result = api.autocomplete().await.unwrap();

    assert!(result.autocomplete.data.contains(&"eieren".into()))
}

#[tokio::test]
pub async fn test_search() {
    let api = conditional_test!(base_jumbo_api);

    let result = api.search("croissant", None, None).await.unwrap();

    assert!(result.products.total > 0)
}
