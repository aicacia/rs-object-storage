use std::path::Path;

use axum::body::Bytes;
use tokio::{fs, io::AsyncWriteExt};

use crate::{
  core::{config::get_config, database::run_transaction},
  repository::{self, file::FileRow},
};

pub async fn create_file(
  pool: &sqlx::AnyPool,
  path: String,
  kind: Option<String>,
) -> sqlx::Result<FileRow> {
  run_transaction(pool, |transaction| {
    Box::pin(async {
      let config = get_config();
      let files_path = Path::new(&config.files_dir);

      let file_row = repository::file::create_file(transaction, path, kind, 0).await?;
      let file_path = files_path.join(file_row.id.to_string());

      let _ = fs::File::create(file_path).await?;

      Ok(file_row)
    })
  })
  .await
}

pub async fn append_file(
  pool: &sqlx::AnyPool,
  file_id: i64,
  file: &mut fs::File,
  bytes: Bytes,
) -> sqlx::Result<usize> {
  let written = bytes.len();
  file.write_all(&bytes).await?;
  let _ = repository::file::update_file_size(pool, file_id, written as i64).await?;
  Ok(written)
}

pub async fn delete_file(pool: &sqlx::AnyPool, file_id: i64) -> sqlx::Result<Option<FileRow>> {
  run_transaction(pool, move |transaction| {
    Box::pin(async move {
      let config = get_config();
      let files_path = Path::new(&config.files_dir);
      let file_path = files_path.join(file_id.to_string());
      let file_row = repository::file::delete_file(transaction, file_id).await?;
      fs::remove_file(file_path).await?;
      Ok(file_row)
    })
  })
  .await
}
