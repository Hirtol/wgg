use crate::db;
use platform_dirs::AppDirs;
use sea_orm::{DatabaseConnection, EntityTrait, PaginatorTrait, TransactionTrait};

pub fn get_app_dirs() -> AppDirs {
    platform_dirs::AppDirs::new("Wgg".into(), false).expect("Couldn't find a home directory for config!")
}

/// Performed so long as the amount of Users in the database is 0, always need at least one!
pub async fn first_time_setup(db: &DatabaseConnection) -> anyhow::Result<()> {
    let tx = db.begin().await?;

    let user_count = db::users::Entity::find().count(&tx).await?;

    if user_count == 0 {
        tracing::info!(
            "No user exists, creating default admin user with (email: `admin@admin.com`, password: `admin`)"
        );

        let user = crate::api::UserCreateInput {
            username: "admin".to_string(),
            email: "admin@admin.com".to_string(),
            password: "admin".to_string(),
            is_admin: true,
        };

        let _ = crate::api::create_user(&tx, user).await?;
    }

    tx.commit().await?;

    Ok(())
}
