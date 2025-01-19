use crate::{db::DbPool, errors::ApiError};
use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse, Result,
};
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as};

#[derive(Debug, Deserialize)]
struct RequestBody {
    texts: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ResponseBody {
    message: String,
}

#[post("/input")]
async fn input(
    pool: Data<DbPool>,
    request_body: Json<RequestBody>,
) -> Result<HttpResponse, ApiError> {
    Ok(HttpResponse::Ok().json(ResponseBody {
        message: "Success Ingesting Data".to_string(),
    }))
}
