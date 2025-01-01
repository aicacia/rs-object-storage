use std::collections::HashMap;

#[derive(Default, sqlx::FromRow)]
pub struct FileRow {
  pub id: i64,
  pub path: String,
  #[sqlx(rename = "type")]
  pub kind: Option<String>,
  pub size: i64,
  pub updated_at: i64,
  pub created_at: i64,
}

impl FileRow {
  pub fn is_dir(&self) -> bool {
    self.id == 0
  }
}

async fn get_files(
  pool: &sqlx::AnyPool,
  path: &str,
  limit: Option<usize>,
  offset: Option<usize>,
) -> sqlx::Result<Vec<FileRow>> {
  let mut qb = sqlx::QueryBuilder::new("SELECT f.* FROM files f");
  if !path.is_empty() {
    qb.push(" WHERE f.path LIKE ")
      .push_bind(format!("{}/%", path));
  }
  if let Some(limit) = limit {
    qb.push(" LIMIT ").push_bind(limit as i64);
  }
  if let Some(offset) = offset {
    qb.push(" OFFSET ").push_bind(offset as i64);
  }
  qb.build_query_as().fetch_all(pool).await
}

pub async fn get_files_and_folders(
  pool: &sqlx::AnyPool,
  path: Option<&str>,
  limit: Option<usize>,
  offset: Option<usize>,
) -> sqlx::Result<Vec<FileRow>> {
  let path = path
    .unwrap_or_default()
    .trim_start_matches("/")
    .trim_end_matches("/");

  let file_rows = get_files(pool, path, limit, offset).await?;
  let path_parts = if path.is_empty() {
    Vec::new()
  } else {
    path.split("/").collect::<Vec<&str>>()
  };
  let path_depth = path_parts.len();

  let mut folders = HashMap::new();
  let mut files = Vec::new();

  for file_row in file_rows {
    let file_parts = file_row.path.split("/").collect::<Vec<&str>>();
    let file_depth = file_parts.len() - 1;

    if file_depth == path_depth {
      files.push(file_row);
      continue;
    }
    let file_folder = file_parts[path_depth..(path_depth + 1)].join("/");
    let folder = folders
      .entry(file_folder.clone())
      .or_insert_with(|| FileRow {
        id: 0,
        path: file_folder,
        kind: Some("directory".to_owned()),
        size: 0,
        updated_at: 0,
        created_at: 0,
      });
    folder.size += file_row.size;
    if folder.updated_at > file_row.updated_at || folder.updated_at == 0 {
      folder.updated_at = file_row.updated_at;
    }
    if folder.created_at < file_row.created_at || folder.created_at == 0 {
      folder.created_at = file_row.created_at;
    }
  }

  Ok(folders.into_values().chain(files).collect())
}

pub async fn get_file_by_path(pool: &sqlx::AnyPool, path: &str) -> sqlx::Result<Option<FileRow>> {
  sqlx::query_as("SELECT f.* FROM files f WHERE f.path = $1")
    .bind(path.trim_start_matches("/").trim_end_matches("/"))
    .fetch_optional(pool)
    .await
}

pub async fn get_file_by_id(pool: &sqlx::AnyPool, id: i64) -> sqlx::Result<Option<FileRow>> {
  sqlx::query_as("SELECT f.* FROM files f WHERE f.id = $1")
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub(crate) async fn create_file(
  transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
  path: String,
  kind: Option<String>,
  size: i64,
) -> sqlx::Result<FileRow> {
  sqlx::query_as("INSERT INTO files (path, type, size) VALUES ($1, $2, $3) RETURNING *")
    .bind(path.trim_start_matches("/").trim_end_matches("/"))
    .bind(kind)
    .bind(size)
    .fetch_one(&mut **transaction)
    .await
}

pub(crate) async fn update_file_size(
  pool: &sqlx::AnyPool,
  id: i64,
  size: i64,
) -> sqlx::Result<Option<FileRow>> {
  sqlx::query_as("UPDATE files SET size = $1 WHERE id = $2 RETURNING *")
    .bind(size)
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub(crate) async fn update_file_path(
  pool: &sqlx::AnyPool,
  id: i64,
  path: String,
  kind: Option<String>,
) -> sqlx::Result<Option<FileRow>> {
  sqlx::query_as("UPDATE files SET path = $1, type = COALESCE($2, type) WHERE id = $3 RETURNING *")
    .bind(path)
    .bind(kind)
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub(crate) async fn delete_file(
  transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
  id: i64,
) -> sqlx::Result<Option<FileRow>> {
  sqlx::query_as("DELETE FROM files WHERE id = $1 RETURNING *")
    .bind(id)
    .fetch_optional(&mut **transaction)
    .await
}
