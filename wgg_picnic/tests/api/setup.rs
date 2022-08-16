use wgg_picnic::config::Config;
use wgg_picnic::{Credentials, PicnicApi};

/// Create an instance for the PicnicApi integration tests.
///
/// This expects a `.env` file with `PICNIC_AUTH_TOKEN` and `PICNIC_USER_ID` to exist for testing the authorised API.
pub fn picnic_api() -> PicnicApi {
    let auth_cred = dotenv::var("PICNIC_AUTH_TOKEN").expect("Expected an environment variable to exist");
    let user_id = dotenv::var("PICNIC_USER_ID").expect("Expected an environment variable to exist");

    let cred = Credentials::new(auth_cred, user_id);

    PicnicApi::new(cred, Config::default())
}
