use async_graphql::dataloader::DataLoader;

pub struct DataLoaders {
    pub backend: DataLoader<()>,
}

impl DataLoaders {
    pub fn new() -> Self {
        DataLoaders {
            backend: DataLoader::new((), tokio::spawn),
        }
    }
}
