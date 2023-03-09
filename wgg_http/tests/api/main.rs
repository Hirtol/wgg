use crate::setup::WggClient;
use wgg_http::api::LoginInput;
use wgg_http::setup::DEFAULT_USER;

mod graphql;
mod setup;

#[tokio::test]
async fn test_login() {
    let app = setup::TestApp::spawn_app().await;
    let (_, id) = WggClient::with_login_and_user_id(app).await;

    // First-time setup user should always have id 1 to start with
    assert_eq!(id, 1)
}

#[tokio::test]
async fn test_http_login_admin() {
    let app = setup::TestApp::spawn_app().await.to_client();
    let input = LoginInput {
        email: DEFAULT_USER.email.clone(),
        password: DEFAULT_USER.password.clone(),
    };

    let response = app.post("/api/auth/login").json(&input).send().await.unwrap();

    assert!(response.status().is_success())
}
