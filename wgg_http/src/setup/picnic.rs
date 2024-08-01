use async_graphql::futures_util::future::BoxFuture;
use std::io;
use std::path::PathBuf;
use std::sync::Arc;
use tracing_subscriber::filter::FilterExt;
use wgg_providers::wgg_picnic::credentials::cache::JsonFileCache;
use wgg_providers::wgg_picnic::credentials::{Credentials, CredentialsCache};

pub struct PicnicCredentialsCache {
    json_cache: JsonFileCache,
}

impl PicnicCredentialsCache {
    pub fn new(json_path: impl Into<PathBuf>) -> Self {
        Self {
            json_cache: JsonFileCache::from_file(json_path),
        }
    }
}

impl CredentialsCache for PicnicCredentialsCache {
    fn request_credentials(&self) -> BoxFuture<wgg_providers::wgg_picnic::Result<Option<Arc<Credentials>>>> {
        self.json_cache.request_credentials()
    }

    fn persist_credentials(
        &mut self,
        credentials: Arc<Credentials>,
    ) -> BoxFuture<wgg_providers::wgg_picnic::Result<()>> {
        self.json_cache.persist_credentials(credentials)
    }

    fn request_2fa_code(&self) -> BoxFuture<wgg_providers::wgg_picnic::Result<String>> {
        use futures::FutureExt;
        async move {
            tracing::warn!("Required to provide a 2FA code from Picnic to login, please enter it into the terminal!");
            let mut buffer = String::new();
            io::stdin().read_line(&mut buffer).map_err(|e| anyhow::anyhow!(e))?;
            trim_newline(&mut buffer);
            Ok(buffer)
        }
        .boxed()
    }
}

fn trim_newline(s: &mut String) {
    if s.ends_with('\n') {
        s.pop();
        if s.ends_with('\r') {
            s.pop();
        }
    }
}
