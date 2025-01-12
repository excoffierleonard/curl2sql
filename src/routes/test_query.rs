use crate::{db::DbPool, errors::ApiError, responses::ApiResponse};
use actix_web::{get, web::Data, Result};
use serde::Serialize;
use sqlx::{query_as, types::chrono::NaiveDateTime};

#[derive(Serialize, Debug)]
struct User {
    id: i32,
    email: String,
    password: String,
    created_at: NaiveDateTime,
}

#[get("/v1/users")]
async fn custom_query(pool: Data<DbPool>) -> Result<ApiResponse<Vec<User>>, ApiError> {
    let users = query_as!(User, "SELECT id, email, password, created_at FROM users;")
        .fetch_all(pool.get_pool())
        .await?;

    Ok(ApiResponse::data(users))
}
