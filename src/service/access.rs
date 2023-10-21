use anyhow::Result;
use sqlx::{Pool, Postgres};
use uuid::Uuid;

use crate::{
  core::encryption::{encrypt_password, random_password, verify_password},
  model::access::{AccessRow, AccessWithExposedSecret},
};

pub async fn get_access_by_id(pool: &Pool<Postgres>, id: &Uuid) -> Result<Option<AccessRow>> {
  Ok(
    sqlx::query_as!(
      AccessRow,
      r#"select a.id, a.encrypted_secret, a.admin, a.created_at, a.updated_at from access a where a.id = $1;"#,
      id,
    )
    .fetch_optional(pool)
    .await?,
  )
}

pub async fn validate_access(
  pool: &Pool<Postgres>,
  id: &Uuid,
  secret: &str,
) -> Result<Option<AccessRow>> {
  if let Some(access) = get_access_by_id(pool, id).await? {
    if verify_password(secret, &access.encrypted_secret)? {
      return Ok(Some(access));
    } else {
      log::debug!("Access secret mismatch");
    }
  } else {
    log::debug!("Access not found");
  }
  Ok(None)
}

pub async fn create_access(pool: &Pool<Postgres>, admin: bool) -> Result<AccessWithExposedSecret> {
  let id = uuid::Uuid::new_v4();
  let secret = random_password(64);
  let encrypted_secret = encrypt_password(&secret)?;
  let access = sqlx::query_as!(
    AccessRow,
    r#"insert into access (id, encrypted_secret, admin)
    values ($1, $2, $3)
    returning id, encrypted_secret, admin, created_at, updated_at;"#,
    id,
    encrypted_secret,
    admin,
  )
  .fetch_one(pool)
  .await?;
  Ok(AccessWithExposedSecret {
    id: access.id,
    secret: secret,
    admin: access.admin,
    created_at: access.created_at,
    updated_at: access.updated_at,
  })
}

pub async fn reset_access(pool: &Pool<Postgres>, id: &Uuid) -> Result<AccessWithExposedSecret> {
  let secret = random_password(64);
  let encrypted_secret = encrypt_password(&secret)?;
  let access = sqlx::query_as!(
    AccessRow,
    r#"update access
    set encrypted_secret = $2
    where id = $1
    returning id, encrypted_secret, admin, created_at, updated_at;"#,
    id,
    encrypted_secret
  )
  .fetch_one(pool)
  .await?;
  Ok(AccessWithExposedSecret {
    id: access.id,
    secret: secret,
    admin: access.admin,
    created_at: access.created_at,
    updated_at: access.updated_at,
  })
}
