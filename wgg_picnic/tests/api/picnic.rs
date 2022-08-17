use crate::common::picnic_api;
use crate::conditional_test;
use wgg_picnic::models::ImageSize;

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

#[tokio::test]
pub async fn test_product() {
    conditional_test!();
    let api = picnic_api();

    let result = api.product("11470254").await.unwrap();
    assert_eq!(result.product_details.name, "Picnic halfvolle melk");
}

#[tokio::test]
pub async fn test_product_image() {
    conditional_test!();
    let api = picnic_api();

    // Halfvolle melk
    let product = api.product("11470254").await.unwrap();
    let result = api
        .image(product.product_details.image_id, ImageSize::Tiny)
        .await
        .unwrap();

    assert!(!result.is_empty())
}
