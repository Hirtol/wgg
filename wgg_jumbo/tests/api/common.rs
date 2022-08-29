use wgg_jumbo::Config;
use wgg_jumbo::{BaseJumboApi, Credentials, FullJumboApi};

/// The environment variable that needs to be set to start live testing.
pub const LIVE_TESTING_ENV: &str = "API_LIVE_TESTING";
pub const LIVE_AUTH_TESTING_ENV: &str = "AUTH_API_LIVE_TESTING";

/// Create an instance for the FullJumboApi integration tests.
///
/// This expects a `.env` file with `JUMBO_AUTH_TOKEN` to exist for testing the authorised API.
pub fn full_jumbo_api() -> FullJumboApi {
    let auth_cred = dotenv::var("JUMBO_AUTH_TOKEN").expect("Expected an environment variable to exist");

    let cred = Credentials::new(auth_cred);

    FullJumboApi::new(cred, Config::default())
}

/// Create an instance for the FullJumboApi integration tests.
///
/// No authorisation si needed
pub fn base_jumbo_api() -> BaseJumboApi {
    BaseJumboApi::new(Default::default())
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

#[macro_export]
macro_rules! auth_conditional_test {
    ($fns:path) => {{
        $crate::conditional_test!($fns, $crate::common::LIVE_AUTH_TESTING_ENV)
    }};
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
