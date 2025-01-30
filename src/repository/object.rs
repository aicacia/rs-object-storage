use std::collections::HashMap;

#[derive(Default, sqlx::FromRow)]
pub struct ObjectRow {
  pub id: i64,
  pub path: String,
  pub r#type: Option<String>,
  pub size: i64,
  pub updated_at: i64,
  pub created_at: i64,
}

impl ObjectRow {
  pub fn is_dir(&self) -> bool {
    self.id == 0
  }
}

async fn get_objects(
  pool: &sqlx::AnyPool,
  path: &str,
  limit: Option<usize>,
  offset: Option<usize>,
) -> sqlx::Result<Vec<ObjectRow>> {
  let mut qb = sqlx::QueryBuilder::new("SELECT f.* FROM objects f");
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

pub async fn get_objects_and_folders(
  pool: &sqlx::AnyPool,
  path: Option<&str>,
  limit: Option<usize>,
  offset: Option<usize>,
) -> sqlx::Result<Vec<ObjectRow>> {
  let path = path
    .unwrap_or_default()
    .trim_start_matches("/")
    .trim_end_matches("/");

  let object_rows = get_objects(pool, path, limit, offset).await?;
  let path_parts = if path.is_empty() {
    Vec::new()
  } else {
    path.split("/").collect::<Vec<&str>>()
  };
  let path_depth = path_parts.len();

  let mut folders = HashMap::new();
  let mut objects = Vec::new();

  for object_row in object_rows {
    let object_parts = object_row.path.split("/").collect::<Vec<&str>>();
    let object_depth = object_parts.len() - 1;

    if object_depth == path_depth {
      objects.push(object_row);
      continue;
    }
    let object_folder = object_parts[path_depth..(path_depth + 1)].join("/");
    let folder = folders
      .entry(object_folder.clone())
      .or_insert_with(|| ObjectRow {
        id: 0,
        path: object_folder,
        r#type: Some("directory".to_owned()),
        size: 0,
        updated_at: 0,
        created_at: 0,
      });
    folder.size += object_row.size;
    if folder.updated_at > object_row.updated_at || folder.updated_at == 0 {
      folder.updated_at = object_row.updated_at;
    }
    if folder.created_at < object_row.created_at || folder.created_at == 0 {
      folder.created_at = object_row.created_at;
    }
  }

  Ok(folders.into_values().chain(objects).collect())
}

pub async fn get_object_by_path(
  pool: &sqlx::AnyPool,
  path: &str,
) -> sqlx::Result<Option<ObjectRow>> {
  sqlx::query_as("SELECT f.* FROM objects f WHERE f.path = $1")
    .bind(path.trim_start_matches("/").trim_end_matches("/"))
    .fetch_optional(pool)
    .await
}

pub async fn get_object_by_id(pool: &sqlx::AnyPool, id: i64) -> sqlx::Result<Option<ObjectRow>> {
  sqlx::query_as("SELECT f.* FROM objects f WHERE f.id = $1")
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn create_object(
  transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
  path: String,
  kind: Option<String>,
  size: i64,
) -> sqlx::Result<ObjectRow> {
  sqlx::query_as("INSERT INTO objects (path, type, size) VALUES ($1, $2, $3) RETURNING *")
    .bind(path.trim_start_matches("/").trim_end_matches("/"))
    .bind(kind)
    .bind(size)
    .fetch_one(&mut **transaction)
    .await
}

pub async fn update_object_size(
  pool: &sqlx::AnyPool,
  id: i64,
  size: i64,
) -> sqlx::Result<Option<ObjectRow>> {
  sqlx::query_as("UPDATE objects SET size = $1 WHERE id = $2 RETURNING *")
    .bind(size)
    .bind(id)
    .fetch_optional(pool)
    .await
}

pub async fn update_object_path(
  pool: &sqlx::AnyPool,
  id: i64,
  path: String,
  kind: Option<String>,
) -> sqlx::Result<Option<ObjectRow>> {
  sqlx::query_as(
    "UPDATE objects SET path = $1, type = COALESCE($2, type) WHERE id = $3 RETURNING *",
  )
  .bind(path)
  .bind(kind)
  .bind(id)
  .fetch_optional(pool)
  .await
}

pub async fn delete_object(
  transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
  id: i64,
) -> sqlx::Result<Option<ObjectRow>> {
  sqlx::query_as("DELETE FROM objects WHERE id = $1 RETURNING *")
    .bind(id)
    .fetch_optional(&mut **transaction)
    .await
}

pub async fn delete_object_by_path(
  transaction: &mut sqlx::Transaction<'_, sqlx::Any>,
  path: &str,
) -> sqlx::Result<Option<ObjectRow>> {
  sqlx::query_as("DELETE FROM objects WHERE path = $1 RETURNING *")
    .bind(path)
    .fetch_optional(&mut **transaction)
    .await
}
