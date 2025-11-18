// use crate::errors::AppError;
// // use crate::models::auth::JwtClaims;
// // use crate::session::SessionData;
// use axum::{
//   extract::{FromRequest, FromRequestParts, Json, Path, Request},
//   http::{request::Parts, StatusCode},
//   response::{IntoResponse, Response},
// };
// use serde::de::DeserializeOwned;
// use uuid::Uuid;
// use validator::Validate;
//
// pub struct UuidPath(pub Uuid);
//
// impl<S> FromRequestParts<S> for UuidPath
// where
//   S: Send + Sync,
// {
//   type Rejection = Response;
//
//   async fn from_request_parts(parts: &mut Parts, state: &S) -> Result<Self, Self::Rejection> {
//     let Path(id) = Path::<String>::from_request_parts(parts, state)
//       .await
//       .map_err(|e| e.into_response())?;
//
//     match Uuid::parse_str(&id) {
//       Ok(uuid) => Ok(UuidPath(uuid)),
//       Err(e) => Err(AppError::UuidError(e).into_response()),
//     }
//   }
// }
//
// pub struct ValidatedJson<T>(pub T);
//
// impl<T, S> FromRequest<S> for ValidatedJson<T>
// where
//   T: DeserializeOwned + Validate,
//   S: Send + Sync,
// {
//   type Rejection = Response;
//
//   async fn from_request(req: Request, state: &S) -> Result<Self, Self::Rejection> {
//     let Json(data) = Json::<T>::from_request(req, state)
//       .await
//       .map_err(|e| e.into_response())?;
//
//     data.validate()
//       .map_err(|e| AppError::ValidationError(e).into_response())?;
//
//     Ok(ValidatedJson(data))
//   }
// }
//
// // pub struct AuthenticatedUser(pub JwtClaims);
// //
// // impl<S> FromRequestParts<S> for AuthenticatedUser
// // where
// //   S: Send + Sync,
// // {
// //   type Rejection = (StatusCode, &'static str);
// //
// //   async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
// //     parts
// //       .extensions
// //       .get::<JwtClaims>()
// //       .cloned()
// //       .map(AuthenticatedUser)
// //       .ok_or((
// //         StatusCode::UNAUTHORIZED,
// //         "Authentication required but no JWT claims found in request. \
// //                  Ensure jwt_redis_auth_middleware is applied to this route.",
// //       ))
// //   }
// // }
//
// /// Extractor for session-based authentication
// ///
// /// Use this in route handlers to require session authentication.
// /// The session middleware must be applied to the route for this to work.
// ///
// /// # Example
// ///
// /// ```ignore
// /// async fn get_profile(SessionUser(session): SessionUser) -> Json<UserProfile> {
// ///     Json(UserProfile {
// ///         email: session.email,
// ///         name: session.name,
// ///     })
// /// }
// /// ```
// // pub struct SessionUser(pub SessionData);
// //
// // impl<S> FromRequestParts<S> for SessionUser
// // where
// //   S: Send + Sync,
// // {
// //   type Rejection = (StatusCode, &'static str);
// //
// //   async fn from_request_parts(parts: &mut Parts, _state: &S) -> Result<Self, Self::Rejection> {
// //     parts
// //       .extensions
// //       .get::<SessionData>()
// //       .cloned()
// //       .map(SessionUser)
// //       .ok_or((
// //         StatusCode::UNAUTHORIZED,
// //         "Authentication required but no session found in request. \
// //                  Ensure session_auth_middleware is applied to this route.",
// //       ))
// //   }
// // }
