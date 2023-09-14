use crate::common::picnic_api;
use crate::conditional_test;
use wgg_picnic::models::{ImageSize, SearchItem};

#[tokio::test]
pub async fn test_user_data() {
    let api = conditional_test!(picnic_api);

    let result = api.user_details().await.unwrap();

    assert_eq!(result.customer_type, "CONSUMER")
}

#[tokio::test]
pub async fn test_search() {
    let api = conditional_test!(picnic_api);

    let result = api.search("melk").await.unwrap();
    // Picnic halfvolle melk, the Vec always seems to have just one item in it.
    let milk_exists = result[0]
        .items
        .iter()
        .filter_map(|x| match x {
            SearchItem::SingleArticle(article) => Some(article),
            _ => None,
        })
        .find(|x| x.id == "11470254");

    assert!(milk_exists.is_some())
}

#[tokio::test]
pub async fn test_suggestion() {
    let api = conditional_test!(picnic_api);

    let result = api.suggestions("melk").await.unwrap();
    // halfvolle melk
    let milk_exists = result.iter().find(|x| x.suggestion == "halfvolle melk");

    assert!(milk_exists.is_some())
}

#[tokio::test]
pub async fn test_product() {
    let api = conditional_test!(picnic_api);

    let result = api.product("11470254").await.unwrap();
    assert_eq!(result.name, "Picnic halfvolle melk");
}

#[tokio::test]
pub async fn test_promotion() {
    let api = picnic_api();

    let result = api.promotions().await.unwrap();

    println!("DATA: {:#?}", result);
    std::fs::write("OUTPUT.txt", format!("{:#?}", result));
}

#[tokio::test]
pub async fn test_product_image() {
    let api = conditional_test!(picnic_api);

    // Halfvolle melk
    let product = api.product("11470254").await.unwrap();
    let result = api.image(&product.images[0].image_id, ImageSize::Tiny).await.unwrap();

    assert!(!result.is_empty())
}
