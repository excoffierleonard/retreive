use crate::{db::DbPool, errors::ApiError};
use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse, Result,
};
use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::query;
use std::env::var;

#[derive(Debug, Deserialize)]
struct RequestBody {
    text: String,
    top_k: i32,
}

#[derive(Debug, Serialize)]
struct ResponseBody {
    texts: Vec<String>,
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

#[post("/fetch_similar")]
async fn fetch_similar(
    pool: Data<DbPool>,
    request_body: Json<RequestBody>,
) -> Result<HttpResponse, ApiError> {
    dotenv().ok();

    let oai_api_key = var("OPENAI_API_KEY")?;

    let embedding_request_body = EmbeddingRequestBody {
        model: "text-embedding-3-large".to_string(),
        texts: vec![request_body.text],
    };

    let embedding_response_body: EmbeddingResponseBody = Client::new()
        .post("https://embedder.excoffierleonard.com/embed")
        .bearer_auth(&oai_api_key)
        .json(&embedding_request_body)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    let most_similar_texts = query(
        "
        SELECT text
        FROM main
        ORDER BY embedding <=> $1::vector(3072)
        LIMIT $2;
        ",
    )
    .bind(embedding_response_body.embeddings[0])
    .bind(request_body.top_k)
    .fetch_all(pool.get_pool())
    .await?;

    Ok(HttpResponse::Ok().json(ResponseBody {
        texts: most_similar_texts,
    }))
}
