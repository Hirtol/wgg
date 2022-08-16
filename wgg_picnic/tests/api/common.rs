use wgg_picnic::config::Config;
use wgg_picnic::{Credentials, PicnicApi};

/// The environment variable that needs to be set to start live testing.
pub const LIVE_TESTING_ENV: &str = "API_LIVE_TESTING";

/// Create an instance for the PicnicApi integration tests.
///
/// This expects a `.env` file with `PICNIC_AUTH_TOKEN` and `PICNIC_USER_ID` to exist for testing the authorised API.
pub fn picnic_api() -> PicnicApi {
    let auth_cred = dotenv::var("PICNIC_AUTH_TOKEN").expect("Expected an environment variable to exist");
    let user_id = dotenv::var("PICNIC_USER_ID").expect("Expected an environment variable to exist");

    let cred = Credentials::new(auth_cred, user_id);

    PicnicApi::new(cred, Config::default())
}

#[macro_export]
macro_rules! function {
    () => {{
        fn f() {}
        fn type_name_of<T>(_: T) -> &'static str {
            std::any::type_name::<T>()
        }
        let name = type_name_of(f);
        &name[..name.len() - 3]
    }};
}

/// Check whether live testing is enabled for this run.
///
/// ```norun
///
/// #[tokio::test]
/// pub async fn test() {
///     conditional_test!();
///
///     ...
/// }
///
/// ```
#[macro_export]
macro_rules! conditional_test {
    () => {
        let is_live = std::env::var($crate::common::LIVE_TESTING_ENV).is_ok();
        let fn_name = $crate::function!();

        if !is_live {
            println!("Skipping: {}", fn_name);
            return;
        }
    };
}
