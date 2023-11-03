use std::{io, path::Path};

use anyhow::Result;
use sha2::{Digest, Sha256};
use sqlx::{Pool, Postgres};

use crate::model::files::FileRow;

pub async fn get_files_and_folders(pool: &Pool<Postgres>, key: &str) -> Result<Vec<FileRow>> {
  let path = if key.ends_with('/') {
    key.to_string()
  } else {
    format!("{}/", key)
  };
  let folders_like = format!("^{}[^/]+/.+$", path);
  let files_like = format!("^{}[^/]+$", path);
  let files = sqlx::query_as!(
    FileRow,
    r#"select 0 as "id!", f.key as "key!", sum(f.size)::integer as "size!", '' as "hash!", max(f.updated_at) as "updated_at!", min(f.created_at) as "created_at!" from (
      select
        concat($1, split_part(f.key, '/', 1)) as key,
        f.size,
        f.updated_at,
        f.created_at
      from (
        select
          substring(f.key, length($1) + 1) as key,
          f.size as size,
          f.updated_at as updated_at,
          f.created_at as created_at
        from
          file f
        where f.key ~ $2
      ) f
    ) f
    group by f.key
    union
    select
      f.id,
      f.key,
      f.size,
      f.hash,
      f.updated_at,
      f.created_at
    from
      file f
    where f.key ~ $3;"#,
    path,
    folders_like,
    files_like,
  )
  .fetch_all(pool)
  .await?;
  Ok(files)
}

pub async fn get_file_by_key(pool: &Pool<Postgres>, key: &str) -> Result<Option<FileRow>> {
  let file = sqlx::query_as!(
    FileRow,
    r#"select
      f.id,
      f.key,
      f.size,
      f.hash,
      f.updated_at,
      f.created_at
    from
      file f
    where f.key = $1;"#,
    key,
  )
  .fetch_optional(pool)
  .await?;
  Ok(file)
}

pub async fn get_file_by_id(pool: &Pool<Postgres>, id: i32) -> Result<Option<FileRow>> {
  let file = sqlx::query_as!(
    FileRow,
    r#"select
      f.id,
      f.key,
      f.size,
      f.hash,
      f.updated_at,
      f.created_at
    from
      file f
    where f.id = $1;"#,
    id,
  )
  .fetch_optional(pool)
  .await?;
  Ok(file)
}

pub fn get_file_key_sha256(key: &str) -> String {
  let mut hasher = Sha256::new();
  hasher.update(key.as_bytes());
  let result = hasher.finalize();
  hex::encode(result)
}

pub async fn copy_file_path_and_hash(
  from: impl AsRef<Path>,
  to: impl AsRef<Path>,
) -> Result<(String, u64)> {
  let mut hasher = Sha256::new();
  let size = copy_file_and_hash(
    &mut hasher,
    tokio::fs::OpenOptions::new()
      .read(true)
      .open(from.as_ref())
      .await?,
    &mut tokio::fs::OpenOptions::new()
      .create(true)
      .truncate(true)
      .write(true)
      .read(true)
      .open(to.as_ref())
      .await?,
  )
  .await?;
  let hash = hex::encode(hasher.finalize());
  Ok((hash, size))
}

// TODO: add a custom copy future that copies files and hashes at the same time
pub async fn copy_file_and_hash(
  hasher: &mut Sha256,
  mut from_file: tokio::fs::File,
  to_file: &mut tokio::fs::File,
) -> Result<u64> {
  let size = tokio::io::copy(&mut from_file, to_file).await?;
  let mut std_to_file = from_file.into_std().await;
  io::copy(&mut std_to_file, hasher)?;
  Ok(size)
}

pub async fn create_file(
  pool: &Pool<Postgres>,
  key: &str,
  hash: &str,
  size: i32,
) -> Result<FileRow> {
  Ok(
    sqlx::query_as!(
      FileRow,
      r#"insert into file (key, hash, size)
        values ($1, $2, $3)
        returning *;"#,
      key,
      hash,
      size
    )
    .fetch_one(pool)
    .await?,
  )
}

pub async fn update_file(
  pool: &Pool<Postgres>,
  key: &str,
  hash: &str,
  size: i32,
) -> Result<FileRow> {
  Ok(
    sqlx::query_as!(
      FileRow,
      r#"update file set hash = $2, size = $3 where key = $1 returning *;"#,
      key,
      hash,
      size
    )
    .fetch_one(pool)
    .await?,
  )
}

pub async fn upsert_file(
  pool: &Pool<Postgres>,
  key: &str,
  hash: &str,
  size: i32,
) -> Result<FileRow> {
  Ok(
    sqlx::query_as!(
      FileRow,
      r#"insert into file (key, hash, size)
        values ($1, $2, $3)
        on conflict (key) do update set hash = $2, size = $3
        returning *;"#,
      key,
      hash,
      size
    )
    .fetch_one(pool)
    .await?,
  )
}
