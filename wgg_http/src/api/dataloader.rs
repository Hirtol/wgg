use async_graphql::dataloader::DataLoader;
use sea_orm::DatabaseConnection;

pub struct DataLoaders {
    _db: DatabaseConnection,
    pub backend: DataLoader<()>,
}

impl DataLoaders {
    pub fn new(db: DatabaseConnection) -> Self {
        DataLoaders {
            _db: db,
            backend: DataLoader::new((), tokio::spawn),
        }
    }
}
