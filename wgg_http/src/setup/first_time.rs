use crate::db;
use once_cell::sync::Lazy;
use sea_orm::{
    ColumnTrait, DatabaseConnection, DatabaseTransaction, EntityTrait, IntoActiveValue, PaginatorTrait, QueryFilter,
    TransactionTrait,
};
use std::collections::HashSet;

/// The initial user that is created in a fresh install of the application.
pub static DEFAULT_USER: Lazy<crate::api::UserCreateInput> = Lazy::new(|| crate::api::UserCreateInput {
    username: "admin".to_string(),
    email: "admin@admin.com".to_string(),
    password: "admin".to_string(),
    is_admin: true,
});

/// Performed so long as the amount of Users in the database is 0, always need at least one!
pub async fn first_time_setup(db: &DatabaseConnection) -> anyhow::Result<()> {
    let tx = db.begin().await?;

    // First check if we need new users
    check_and_create_user(&tx).await?;
    // Then initialise providers.
    initialise_providers(&tx).await?;

    tx.commit().await?;

    Ok(())
}

/// Initialise the `providers` Database table with all current providers.
///
/// This is not done in a migration to prevent them from being forgotten. Doing it this way will ensure
/// that whenever a new provider is added (or removed!) the database is appropriately updated.
async fn initialise_providers(tx: &DatabaseTransaction) -> anyhow::Result<()> {
    let all_providers: HashSet<String> = db::providers::all_providers().collect();
    let current_providers = db::providers::Entity::find().all(tx).await?;
    let current_prov_hash = current_providers.into_iter().map(|p| p.name).collect::<HashSet<_>>();

    // All items in all_providers, but not in the DB should be added in
    let to_add = all_providers
        .difference(&current_prov_hash)
        .map(|name| db::providers::ActiveModel {
            id: Default::default(),
            name: name.clone().into_active_value(),
        })
        .collect::<Vec<_>>();
    let to_remove = current_prov_hash.difference(&all_providers);

    if !to_add.is_empty() {
        let _ = db::providers::Entity::insert_many(to_add).exec(tx).await?;
    }

    let deleted = db::providers::Entity::delete_many()
        .filter(db::providers::Column::Name.is_in(to_remove.map::<&str, _>(|item| item.as_ref())))
        .exec(tx)
        .await?;

    if deleted.rows_affected > 0 {
        tracing::info!(
            deleted_quantity = deleted.rows_affected,
            "Deleted providers and associated data"
        );
    }

    Ok(())
}

async fn check_and_create_user(tx: &DatabaseTransaction) -> anyhow::Result<()> {
    let user_count = db::users::Entity::find().count(tx).await?;

    if user_count == 0 {
        let user = DEFAULT_USER.clone();

        tracing::info!(
            "No user exists, creating default admin user with (email: `{}`, password: `{}`)",
            user.email,
            user.password
        );

        let _ = crate::api::create_user(tx, user).await?;
    }

    Ok(())
}
