use sea_orm::{
    ActiveModelTrait, ActiveValue::*, ColumnTrait, DatabaseConnection, EntityTrait,
    IntoActiveModel, QueryFilter,
};

use crate::entities::users;
use crate::error::AppError;
use crate::utils::now_ms;

// ---------------------------------------------------------------------------
// Queries
// ---------------------------------------------------------------------------

/// Find a user by primary key. Returns `not_found` if missing.
pub async fn find_by_id(
    db: &DatabaseConnection,
    id: i32,
) -> Result<users::Model, AppError> {
    users::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("User not found"))
}

/// Find a user by username. Returns `None` if no match.
pub async fn find_by_username(
    db: &DatabaseConnection,
    username: &str,
) -> Result<Option<users::Model>, AppError> {
    let user = users::Entity::find()
        .filter(users::Column::Username.eq(username))
        .one(db)
        .await?;
    Ok(user)
}

/// Find a user by email. Returns `None` if no match.
pub async fn find_by_email(
    db: &DatabaseConnection,
    email: &str,
) -> Result<Option<users::Model>, AppError> {
    let user = users::Entity::find()
        .filter(users::Column::Email.eq(email))
        .one(db)
        .await?;
    Ok(user)
}

/// List all users.
pub async fn list_all(
    db: &DatabaseConnection,
) -> Result<Vec<users::Model>, AppError> {
    let all = users::Entity::find().all(db).await?;
    Ok(all)
}

// ---------------------------------------------------------------------------
// Mutations
// ---------------------------------------------------------------------------

/// Create a new user. Returns `conflict` if the username is already taken.
pub async fn create(
    db: &DatabaseConnection,
    username: &str,
    password_hash: &str,
    email: Option<&str>,
    privs: &str,
    scrobbling_enabled: bool,
    max_bit_rate: i32,
) -> Result<users::Model, AppError> {
    // Check username uniqueness.
    let existing = users::Entity::find()
        .filter(users::Column::Username.eq(username))
        .one(db)
        .await?;
    if existing.is_some() {
        return Err(AppError::conflict("Username already exists"));
    }

    let now = now_ms();

    let new_user = users::ActiveModel {
        id: NotSet,
        username: Set(username.to_string()),
        password_hash: Set(password_hash.to_string()),
        email: Set(email.map(String::from)),
        privs: Set(privs.to_string()),
        scrobbling_enabled: Set(scrobbling_enabled as i32),
        max_bit_rate: Set(max_bit_rate),
        is_enabled: Set(1),
        created_at: Set(now),
        updated_at: Set(now),
    };

    let result = users::Entity::insert(new_user).exec(db).await?;
    let inserted = users::Entity::find_by_id(result.last_insert_id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::internal("Failed to retrieve created user"))?;

    Ok(inserted)
}

/// Update the current user's own profile. Only the provided fields are changed.
/// `email` accepts three states:
/// - `None` — leave unchanged
/// - `Some(None)` — clear the email
/// - `Some(Some(v))` — set to `v`
pub async fn update_profile(
    db: &DatabaseConnection,
    id: i32,
    email: Option<Option<String>>,
    scrobbling_enabled: Option<bool>,
    max_bit_rate: Option<i32>,
) -> Result<users::Model, AppError> {
    let target = users::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("User not found"))?;

    let mut active_model = target.into_active_model();

    if let Some(email) = email {
        active_model.email = Set(email);
    }
    if let Some(scrobbling_enabled) = scrobbling_enabled {
        active_model.scrobbling_enabled = Set(scrobbling_enabled as i32);
    }
    if let Some(max_bit_rate) = max_bit_rate {
        active_model.max_bit_rate = Set(max_bit_rate);
    }

    active_model.updated_at = Set(now_ms());

    let updated = active_model.update(db).await?;
    Ok(updated)
}

/// Admin update for any user's profile. Semantics match `update_profile`.
pub async fn admin_update(
    db: &DatabaseConnection,
    id: i32,
    email: Option<Option<String>>,
    scrobbling_enabled: Option<bool>,
    max_bit_rate: Option<i32>,
) -> Result<users::Model, AppError> {
    let target = users::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("User not found"))?;

    let mut active_model = target.into_active_model();

    if let Some(email) = email {
        active_model.email = Set(email);
    }
    if let Some(scrobbling_enabled) = scrobbling_enabled {
        active_model.scrobbling_enabled = Set(scrobbling_enabled as i32);
    }
    if let Some(max_bit_rate) = max_bit_rate {
        active_model.max_bit_rate = Set(max_bit_rate);
    }

    active_model.updated_at = Set(now_ms());

    let updated = active_model.update(db).await?;
    Ok(updated)
}

/// Set a user's password hash.
pub async fn set_password(
    db: &DatabaseConnection,
    id: i32,
    password_hash: &str,
) -> Result<(), AppError> {
    let target = users::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("User not found"))?;

    let mut active_model = target.into_active_model();
    active_model.password_hash = Set(password_hash.to_string());
    active_model.updated_at = Set(now_ms());
    active_model.update(db).await?;

    Ok(())
}

/// Set a user's privileges (stored as a JSON string).
pub async fn set_privs(
    db: &DatabaseConnection,
    id: i32,
    privs: &str,
) -> Result<users::Model, AppError> {
    let target = users::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("User not found"))?;

    let mut active_model = target.into_active_model();
    active_model.privs = Set(privs.to_string());
    active_model.updated_at = Set(now_ms());
    let updated = active_model.update(db).await?;

    Ok(updated)
}

/// Enable or disable a user account.
pub async fn set_enabled(
    db: &DatabaseConnection,
    id: i32,
    enabled: bool,
) -> Result<(), AppError> {
    let target = users::Entity::find_by_id(id)
        .one(db)
        .await?
        .ok_or_else(|| AppError::not_found("User not found"))?;

    let mut active_model = target.into_active_model();
    active_model.is_enabled = Set(enabled as i32);
    active_model.updated_at = Set(now_ms());
    active_model.update(db).await?;

    Ok(())
}

/// Delete a user by ID. Succeeds even if the user does not exist.
pub async fn delete(
    db: &DatabaseConnection,
    id: i32,
) -> Result<(), AppError> {
    users::Entity::delete_by_id(id).exec(db).await?;
    Ok(())
}
