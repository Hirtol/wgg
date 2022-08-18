use crate::common::base_jumbo_api;
use crate::conditional_test;
use wgg_jumbo::clients::BaseApi;

#[tokio::test]
pub async fn test_promotion_tabs() {
    let api = conditional_test!(base_jumbo_api);

    let result = api.promotion_tabs().await.unwrap();

    assert!(result.tabs.iter().any(|tab| tab.id == "weekaanbiedingen"))
}
