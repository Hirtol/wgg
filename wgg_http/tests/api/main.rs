use crate::setup::WggClient;

mod graphql;
mod setup;

#[tokio::test]
async fn test_login() {
    let app = setup::TestApp::spawn_app().await;
    let (_, id) = WggClient::with_login_and_user_id(app).await;

    // First-time setup user should always have id 1 to start with
    assert_eq!(id, 1)
}
