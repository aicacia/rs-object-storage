use std::path::Path;

use axum::body::Bytes;
use tokio::{fs, io::AsyncWriteExt};

use crate::{
  core::{config::get_config, database::run_transaction},
  repository::{self, object::ObjectRow},
};

pub async fn create_object(
  pool: &sqlx::AnyPool,
  path: String,
  kind: Option<String>,
) -> sqlx::Result<ObjectRow> {
  run_transaction(pool, |transaction| {
    Box::pin(async {
      let config = get_config();
      let objects_path = Path::new(&config.objects_dir);

      let object_row = repository::object::create_object(transaction, path, kind, 0).await?;
      let object_path = objects_path.join(object_row.id.to_string());

      let _ = fs::File::create(object_path).await?;

      Ok(object_row)
    })
  })
  .await
}

pub async fn append_object(
  pool: &sqlx::AnyPool,
  object_id: i64,
  object: &mut fs::File,
  bytes: Bytes,
) -> sqlx::Result<usize> {
  let written = bytes.len();
  object.write_all(&bytes).await?;
  let _ = repository::object::update_object_size(pool, object_id, written as i64).await?;
  Ok(written)
}

pub async fn delete_object(
  pool: &sqlx::AnyPool,
  object_id: i64,
) -> sqlx::Result<Option<ObjectRow>> {
  run_transaction(pool, move |transaction| {
    Box::pin(async move {
      let config = get_config();
      let objects_path = Path::new(&config.objects_dir);
      let object_path = objects_path.join(object_id.to_string());
      let object_row = repository::object::delete_object(transaction, object_id).await?;
      fs::remove_file(object_path).await?;
      Ok(object_row)
    })
  })
  .await
}
