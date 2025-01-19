use crate::{db::DbPool, errors::ApiError};
use actix_web::{
    post,
    web::{Data, Json},
    HttpResponse, Result,
};
use dotenv::dotenv;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use sqlx::QueryBuilder;
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

    let embedding_response_body: EmbeddingResponseBody = Client::new()
        .post("https://embedder.excoffierleonard.com/embed")
        .bearer_auth(&oai_api_key)
        .json(&embedding_request_body)
        .send()
        .await?
        .error_for_status()?
        .json()
        .await?;

    // Create a query builder for bulk insert
    let mut query_builder = QueryBuilder::new("INSERT INTO main (text, embedding) ");

    // Start the VALUES clause
    query_builder.push_values(
        request_body
            .texts
            .iter()
            .zip(embedding_response_body.embeddings.iter()),
        |mut b, (text, embedding)| {
            b.push_bind(text).push_bind(embedding);
        },
    );

    // Do nothing if the text already exists
    query_builder.push(" ON CONFLICT (text) DO NOTHING");

    // Execute the bulk insert query
    query_builder.build().execute(pool.get_pool()).await?;

    Ok(HttpResponse::Ok().json(ResponseBody {
        message: "Success".to_string(),
    }))
}
