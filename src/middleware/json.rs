use axum::extract::{rejection::JsonRejection, FromRequest, Request};

use crate::core::error::{InternalError, REQUEST_BODY};

pub struct Json<T>(pub T);

impl<S, T> FromRequest<S> for Json<T>
where
  axum::Json<T>: FromRequest<S, Rejection = JsonRejection>,
  S: Send + Sync,
{
  type Rejection = InternalError;

  async fn from_request(request: Request, state: &S) -> Result<Self, Self::Rejection> {
    let axum::Json(value) = match axum::Json::<T>::from_request(request, state).await {
      Ok(value) => value,
      Err(rejection) => {
        return Err(InternalError::bad_request().with_error(REQUEST_BODY, rejection.to_string()));
      }
    };
    Ok(Self(value))
  }
}
