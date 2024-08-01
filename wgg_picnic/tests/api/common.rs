use std::sync::Arc;
use wgg_picnic::credentials::cache::MemoryCache;
use wgg_picnic::credentials::Credentials;
use wgg_picnic::PicnicApi;
use wgg_picnic::{Config, LoginCredentials};

/// The environment variable that needs to be set to start live testing.
pub const LIVE_TESTING_ENV: &str = "AUTH_API_LIVE_TESTING";

/// Create an instance for the PicnicApi integration tests.
///
/// This expects a `.env` file with `PICNIC_AUTH_TOKEN` and `PICNIC_USER_ID` to exist for testing the authorised API.
pub fn picnic_api() -> PicnicApi {
    let auth_cred = dotenv::var("PICNIC_AUTH_TOKEN").expect("Expected an environment variable to exist");
    let user_id = dotenv::var("PICNIC_USER_ID").expect("Expected an environment variable to exist");

    let cred = Credentials::new(auth_cred, user_id);
    let cache = MemoryCache::new(Some(Arc::new(cred)));

    PicnicApi::new(cache, Config::default(), LoginCredentials::default())
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
/// If it is enabled, the provided function will be called and the result returned.
///
/// # Example
///
/// ```norun
///
/// pub fn get_api() -> Api {
///     ...
/// }
///
/// #[tokio::test]
/// pub async fn test() {
///     conditional_test!(get_api);
///
///     ...
/// }
///
/// ```
#[macro_export]
macro_rules! conditional_test {
    ($fns:path, $name:expr) => {{
        let is_live = std::env::var($name).is_ok();
        let fn_name = $crate::function!();

        if !is_live {
            println!(
                "Skipping: {} - Set the environment variable {}=() to enable",
                fn_name, $name
            );
            return;
        }

        $fns()
    }};
    ($fns:path) => {{
        $crate::conditional_test!($fns, $crate::common::LIVE_TESTING_ENV)
    }};
}
