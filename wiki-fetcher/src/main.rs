use clap::Parser;
use futures::stream::{self, StreamExt};
use reqwest;
use serde_json::{json, Value};
use std::error::Error;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tokio::sync::{Mutex, Semaphore};

#[derive(Parser)]
#[command(name = "wiki-fetcher")]
#[command(about = "Fetches random Wikipedia articles in parallel and sends them in batches")]
struct Args {
    #[arg(long, default_value_t = 1000)]
    total_size: usize,

    #[arg(long, default_value_t = 50)]
    batch_size: usize,
}

async fn fetch_article(client: &reqwest::Client) -> Result<String, Box<dyn Error>> {
    let response = client
        .get("https://en.wikipedia.org/api/rest_v1/page/random/summary")
        .send()
        .await?;

    let data: Value = response.json().await?;
    if let Some(extract) = data["extract"].as_str() {
        Ok(extract.to_string())
    } else {
        Err("No extract found".into())
    }
}

async fn send_batch(
    client: &reqwest::Client,
    texts: Vec<String>,
    batch_number: usize,
) -> Result<(), Box<dyn Error>> {
    let start = Instant::now();

    let response = client
        .post("http://localhost:8080/v1/input")
        .json(&json!({
            "texts": texts
        }))
        .send()
        .await?;

    let status = response.status();
    let response_text = response.text().await?;
    let duration = start.elapsed();

    println!(
        "Batch {}: Status: {}, Time: {:.2}s, Response: {}",
        batch_number,
        status,
        duration.as_secs_f64(),
        response_text
    );

    Ok(())
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    let args = Args::parse();
    println!(
        "Starting with total_size: {}, batch_size: {}",
        args.total_size, args.batch_size
    );

    // Create rate-limited client
    let client = Arc::new(
        reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()?,
    );

    // Create a semaphore to limit concurrent requests (200 per second max)
    let semaphore = Arc::new(Semaphore::new(200));

    // Shared vector for collecting texts
    let texts = Arc::new(Mutex::new(Vec::with_capacity(args.batch_size)));
    let batch_count = Arc::new(Mutex::new(1));

    // Process articles in parallel with rate limiting
    let futures = stream::iter(0..args.total_size)
        .map(|_| {
            let client = client.clone();
            let semaphore = semaphore.clone();
            let texts = texts.clone();
            let batch_count = batch_count.clone();

            async move {
                // Acquire semaphore permit
                let _permit = semaphore.acquire().await.unwrap();

                // Fetch article
                match fetch_article(&client).await {
                    Ok(text) => {
                        let mut texts_guard = texts.lock().await;
                        texts_guard.push(text);

                        // If we've reached batch size, send the batch
                        if texts_guard.len() >= args.batch_size {
                            let batch_to_send = texts_guard.clone();
                            texts_guard.clear();

                            let current_batch = {
                                let mut count = batch_count.lock().await;
                                let batch_num = *count;
                                *count += 1;
                                batch_num
                            };

                            if let Err(e) = send_batch(&client, batch_to_send, current_batch).await
                            {
                                eprintln!("Error sending batch {}: {}", current_batch, e);
                            }
                        }
                    }
                    Err(e) => eprintln!("Error fetching article: {}", e),
                }

                // Release rate limit after 5ms (200 requests per second)
                tokio::time::sleep(Duration::from_millis(5)).await;
            }
        })
        .buffer_unordered(200) // Process up to 200 concurrent requests
        .collect::<Vec<_>>();

    // Wait for all fetches to complete
    futures.await;

    // Send any remaining texts
    let remaining_texts = {
        let mut texts_guard = texts.lock().await;
        if !texts_guard.is_empty() {
            let batch_to_send = texts_guard.clone();
            texts_guard.clear();
            Some(batch_to_send)
        } else {
            None
        }
    };

    if let Some(final_batch) = remaining_texts {
        let final_batch_num = *batch_count.lock().await;
        if let Err(e) = send_batch(&client, final_batch, final_batch_num).await {
            eprintln!("Error sending final batch {}: {}", final_batch_num, e);
        }
    }

    Ok(())
}
