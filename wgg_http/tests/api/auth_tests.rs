use crate::setup::{TestApp, WggClient};
use once_cell::sync::Lazy;
use wgg_http::api::{create_user, AuthContext, GraphqlResult, LoginInput, UserCreateInput};
use wgg_http::setup::DEFAULT_USER;

#[tokio::test]
async fn test_graphql_login() {
    let app = TestApp::spawn_app().await;
    let (_, id) = WggClient::with_login_and_user_id(app).await;

    // First-time setup user should always have id 1 to start with
    assert_eq!(id, 1)
}

#[tokio::test]
async fn test_http_login_normal() {
    let client = TestApp::spawn_app().await.into_client();
    let _ = create_normal_user(&client.app).await.unwrap();
    let input = LoginInput {
        email: NORMAL_USER.email.clone(),
        password: NORMAL_USER.password.clone(),
    };

    let response = client.post("/api/auth/login").json(&input).send().await.unwrap();

    assert!(response.status().is_success());

    let user = response.json::<AuthContext>().await.unwrap();

    assert_eq!(user.email, NORMAL_USER.email);
}

#[tokio::test]
async fn test_http_login_admin() {
    let app = TestApp::spawn_app().await.into_client();
    let input = LoginInput {
        email: DEFAULT_USER.email.clone(),
        password: DEFAULT_USER.password.clone(),
    };

    let response = app.post("/api/auth/login").json(&input).send().await.unwrap();

    assert!(response.status().is_success());
}
//test unsuccesful admin login using the credentials of a normal user
#[tokio::test]
async fn test_http_login_admin_unsuccessful() {
    let app = TestApp::spawn_app().await.into_client();
    let input = LoginInput {
        email: DEFAULT_USER.email.clone(),
        password: "wrong".to_string(),
    };

    let response = app.post("/api/auth/login").json(&input).send().await.unwrap();

    assert!(response.status().is_client_error() || response.status().is_server_error());
}
//test unsuccesful normal login using the credentials of a normal user
#[tokio::test]
async fn test_http_login_normal_unsuccessfull() {
    let client = TestApp::spawn_app().await.into_client();
    let _ = create_normal_user(&client.app).await.unwrap();
    let input = LoginInput {
        email: NORMAL_USER.email.clone(),
        password: "wrong".to_string(),
    };

    let response = client.post("/api/auth/login").json(&input).send().await.unwrap();

    assert!(response.status().is_client_error() || response.status().is_server_error());

    //let user = response.json::<AuthContext>().await.unwrap();

   // assert_eq!(user.email, NORMAL_USER.email);
}

#[tokio::test]
async fn test_http_register_normal_successful() {
    let client = TestApp::spawn_app().await.into_client();
    let input = LoginInput {
        email: NORMAL_USER.email.clone(),
        password: NORMAL_USER.password.clone(),
    };

    let response = client.post("/api/auth/register").json(&input).send().await.unwrap();

    assert!(response.status().is_success());
}

#[tokio::test]
async fn test_http_register_normal_twice_unsuccessful() {
    let client = TestApp::spawn_app().await.into_client();
    let input = LoginInput {
        email: NORMAL_USER.email.clone(),
        password: NORMAL_USER.password.clone(),
    };

    let first_response = client.post("/api/auth/register").json(&input).send().await.unwrap();

    assert!(response.status().is_success());

    let second_response = client.post("/api/auth/register").json(&input).send().await.unwrap();

    assert!(response.status().is_client_error() || response.status().is_server_error());
}

#[tokio::test]
async fn test_http_register_invalid_email_unsuccessful() {
    let client = TestApp::spawn_app().await.into_client();
    let input = LoginInput {
        email: "emailwithoutatsign.com".to_string(),
        password: NORMAL_USER.password.clone(),
    };

    let response = client.post("/api/auth/register").json(&input).send().await.unwrap();

    assert!(response.status().is_client_error() || response.status().is_server_error());
}

static NORMAL_USER: Lazy<UserCreateInput> = Lazy::new(|| UserCreateInput {
    username: "normal_user".to_string(),
    email: "normal@normal.com".to_string(),
    password: "normal".to_string(),
    is_admin: false,
});
static ADMIN_USER: Lazy<UserCreateInput> = Lazy::new(|| UserCreateInput {
    username: "admin_user".to_string(),
    email: "admin@admin.com".to_string(),
    password: "admin".to_string(),
    is_admin: true,
});

async fn create_normal_user(app: &TestApp) -> GraphqlResult<AuthContext> {
    create_test_user(app, NORMAL_USER.clone()).await
}

async fn create_admin_user(app: &TestApp) -> GraphqlResult<AuthContext> {
    create_test_user(app, ADMIN_USER.clone()).await
}


async fn create_test_user(app: &TestApp, input: UserCreateInput) -> GraphqlResult<AuthContext> {
    create_user(&app.db_pool, input).await
}
#[tokio::test]
async fn test_http_login_invalid_normal() {
    let client = TestApp::spawn_app().await.into_client();
    let _ = create_normal_user(&client.app).await.unwrap();
    let input = LoginInput {
        email: "randominvalid@normal.com".to_string(),
        password: NORMAL_USER.password.clone(),
    };

    let response = client.post("/api/auth/login").json(&input).send().await.unwrap();

    assert!(response.status().is_success());

    let user = response.json::<AuthContext>().await.unwrap();

    assert!(response.status().is_client_error() || response.status().is_server_error());
}


#[tokio::test]
async fn test_http_login_invalid_admin() {
    let app = TestApp::spawn_app().await.into_client();

    let input = LoginInput {
        email: "randomadmininvalid@normal.com".to_string(),
        password: DEFAULT_USER.password.clone(),
    };

    let response = app.post("/api/auth/login").json(&input).send().await.unwrap();

    assert!(response.status().is_client_error() || response.status().is_server_error());
}
#[tokio::test]
async fn test_http_create_admin_user() {
    let client = TestApp::spawn_app().await.into_client();

    // Create a new user
    let response = client
        .post("/api/users")
        .json(&ADMIN_USER.clone())
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());

    // Get the created user's ID from the response body
    let user = response.json::<AuthContext>().await.unwrap();
    let user_id = user.id;

    // Get the user from the database and check that it matches the input
    let response = client
        .get(&format!("/api/users/{}", user_id))
        .send()
        .await
        .unwrap();
    assert!(response.status().is_success());

    let user = response.json::<AuthContext>().await.unwrap();
    assert_eq!(user.username, ADMIN_USER.username);
    assert_eq!(user.email, ADMIN_USER.email);
    assert_eq!(user.is_admin, true);
}
