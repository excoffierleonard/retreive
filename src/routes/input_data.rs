use crate::{db::DbPool, errors::ApiError};
use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse, Result,
};
use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::{query, query_as};
use std::env::var;

#[derive(Debug, Deserialize)]
struct RequestBody {
    texts: Vec<String>,
}

#[derive(Debug, Serialize)]
struct ResponseBody {
    message: String,
}

#[derive(Debug, Serialize)]
struct EmbeddingRequestBody {
    model: String,
    texts: Vec<String>,
}

#[derive(Debug, Deserialize)]
struct EmbeddingResponseBody {
    embeddings: Vec<Vec<f32>>,
}

#[post("/input")]
async fn input(
    pool: Data<DbPool>,
    request_body: Json<RequestBody>,
) -> Result<HttpResponse, ApiError> {
    dotenv().ok();

    let oai_api_key = var("OPENAI_API_KEY")?;

    let embedding_request_body = EmbeddingRequestBody {
        model: "text-embedding-3-large".to_string(),
        texts: request_body.texts.clone(),
    };

    let embeddings: EmbeddingResponseBody = Client::new()
        .post("https://embedder.excoffierleonard.com/embed")
        .bearer_auth(&oai_api_key)
        .json(&embedding_request_body)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    Ok(HttpResponse::Ok().json(ResponseBody {
        message: format!("{:?}", embeddings),
    }))
}
