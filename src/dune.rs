use std::env;

use reqwest::Client;
use serde::{Deserialize, Serialize};

pub async fn fetch_users() -> Result<Vec<String>, anyhow::Error> {
    let mut wallets: Vec<String> = Vec::new();
    let mut next_uri = format!(
        "https://api.dune.com/api/v1/query/{}/results?limit={}",
        env::var("DUNE_QUERY_ID")?,
        env::var("DUNE_LINES_PER_REQUEST")?
    );
    let client = Client::new();

    loop {
        let result = client
            .get(&next_uri)
            .header("X-Dune-API-Key", env::var("DUNE_API_KEY")?)
            .send()
            .await?
            .text()
            .await?;
        let result: DuneResponse =
            serde_json::from_str(&result).map_err(|_| anyhow::Error::msg(result))?;
        let mut batch: Vec<String> = result
            .result
            .rows
            .iter()
            .map(|row| match &row.from {
                Some(from) => from.to_string(),
                None => "".to_string(),
            })
            .collect();
        wallets.append(&mut batch);
        match result.next_uri {
            Some(uri) => next_uri = uri,
            None => break,
        }
        tokio::time::sleep(tokio::time::Duration::from_secs(2)).await;
    }

    Ok(wallets
        .into_iter()
        .collect::<std::collections::HashSet<String>>()
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect())
}

#[derive(Deserialize, Serialize)]
struct DuneResponse {
    execution_id: String,
    query_id: u64,
    is_execution_finished: bool,
    state: String,
    submitted_at: String,
    expires_at: String,
    execution_started_at: String,
    execution_ended_at: String,
    result: DuneResult,
    next_uri: Option<String>,
    next_offset: Option<u64>,
}

#[derive(Deserialize, Serialize)]
struct DuneResult {
    rows: Vec<Rows>,
    metadata: DuneMetadata,
}

#[derive(Deserialize, Serialize)]
struct Rows {
    from: Option<String>,
}

#[derive(Deserialize, Serialize)]
struct DuneMetadata {
    column_names: Vec<String>,
    row_count: u64,
    result_set_bytes: u64,
    total_row_count: u64,
    total_result_set_bytes: u64,
    datapoint_count: u64,
    pending_time_millis: u64,
    execution_time_millis: u64,
}
