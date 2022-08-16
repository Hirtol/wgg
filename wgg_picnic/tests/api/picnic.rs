use crate::setup::picnic_api;

#[tokio::test]
pub async fn test_user_data() {
    let api = picnic_api();

    let result = api.user_details().await.unwrap();

    assert_eq!(result.customer_type, "CONSUMER")
}
