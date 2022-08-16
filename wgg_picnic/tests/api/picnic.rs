use crate::common::picnic_api;
use crate::conditional_test;

#[tokio::test]
pub async fn test_user_data() {
    conditional_test!();
    let api = picnic_api();

    let result = api.user_details().await.unwrap();

    assert_eq!(result.customer_type, "CONSUMER")
}

#[tokio::test]
pub async fn test_search() {
    conditional_test!();
    let api = picnic_api();

    let result = api.search("melk").await.unwrap();
    // Picnic halfvolle melk
    let milk_exists = result.iter().find(|x| x.id == "11470254");
    assert!(milk_exists.is_some())
}

#[tokio::test]
pub async fn test_suggestion() {
    conditional_test!();
    let api = picnic_api();

    let result = api.suggestions("melk").await.unwrap();
    // halfvolle melk
    let milk_exists = result.iter().find(|x| x.suggestion == "halfvolle melk");
    assert!(milk_exists.is_some())
}
