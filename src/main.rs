use std::env;

use file::write_csv;
use log::{error, info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::file::{read_wallets, write_wallets};

mod file;

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    info!("Starting Zircuit wallet fetcher");
    let client = Client::new();
    let users = match read_wallets() {
        Ok(users) => users,
        Err(_) => {
            warn!("No wallets found, fetching from Dune API");
            let users = fetch_users(&client).await.unwrap();
            write_wallets(&users).unwrap();
            users
        }
    };
    let users = users
        .into_iter()
        .filter(|u| {
            u != "0x7a493be5c2ce014cd049bf178a1ac0db1b434744"
                && u != "0x34349c5569e7b846c3558961552d2202760a9789"
                && u != "0xd7df7e085214743530aff339afc420c7c720bfa7"
                && u != "0x0000000000000000000000000000000000000000"
        })
        .collect::<Vec<String>>();
    let total_users = users.len();
    let mut fetched_users = 0;
    let timer = std::time::Instant::now();
    info!("Total users: {}", total_users);

    let mut user_infos = Vec::new();

    // Increasing chunk size causes rate limiting error
    let chunk_size =
        env::var("ZIRCUIT_BATCH_SIZE").unwrap_or("25".to_string()).parse::<usize>().unwrap();

    for users_chunk in users.chunks(chunk_size) {
        let mut handles = Vec::new();
        for user in users_chunk {
            let client = client.clone();
            let user_cl = user.clone();
            let handle = tokio::spawn(async move { fetch_user_info(&client, &user_cl).await });
            handles.push((user, handle));
            // For some reason smaller numbers take longer time by 2 seconds / 250 requests
            tokio::time::sleep(tokio::time::Duration::from_millis(
                env::var("ZIRCUIT_COOLDOWN").unwrap_or("50".to_string()).parse::<u64>().unwrap(),
            ))
            .await;
        }
        for (user, handle) in handles {
            let user_info = handle
                .await
                .unwrap()
                .map_err(|e| {
                    error!("Error fetching user info for {}: {:?}", user, e);
                })
                .unwrap();
            user_infos.push(user_info);
            fetched_users += 1;
            if fetched_users % 250 == 0 {
                info!("Fetched {}/{}", fetched_users, total_users);
                info!("Elapsed time: {:?}", timer.elapsed());
                write_csv(&user_infos).unwrap();
            }
        }
    }
    info!("Finished fetching all users!");
    info!("Elapsed time: {:?}", timer.elapsed());
    write_csv(&user_infos).unwrap();
}

async fn fetch_user_info(client: &Client, address: &str) -> Result<User, anyhow::Error> {
    let user_response = client
        .get(format!("https://stake.zircuit.com/api/user/{}", address))
        .send()
        .await?
        .json::<UserResponse>()
        .await;
    let points_response = client
        .get(format!("https://stake.zircuit.com/api/points/{}", address))
        .send()
        .await?
        .json::<PointsResponse>()
        .await;

    let user: UserResponse = match user_response {
        Ok(user) => user,
        _ => UserResponse {
            referral_code: "".to_string(),
            signed: false,
            signed_build_and_earn: false,
        },
    };

    let points: PointsResponse = match points_response {
        Ok(points) => points,
        _ => PointsResponse {
            total_points: "0".to_string(),
            total_ref_points: "0".to_string(),
            total_build_points: "0".to_string(),
            total_extra_points: "0".to_string(),
            total_pendle_points: "0".to_string(),
        },
    };

    Ok(User {
        address: address.to_string(),
        referral_code: user.referral_code,
        signed: user.signed,
        signed_build_and_earn: user.signed_build_and_earn,
        total_points: points.total_points.parse()?,
        total_ref_points: points.total_ref_points.parse()?,
        total_build_points: points.total_build_points.parse()?,
        total_extra_points: points.total_extra_points.parse()?,
        total_pendle_points: points.total_pendle_points.parse()?,
    })
}

#[derive(Deserialize, Serialize, Debug)]
struct User {
    address: String,
    referral_code: String,
    signed_build_and_earn: bool,
    signed: bool,
    total_points: f64,
    total_ref_points: f64,
    total_build_points: f64,
    total_extra_points: f64,
    total_pendle_points: f64,
}

async fn fetch_users(client: &Client) -> Result<Vec<String>, anyhow::Error> {
    let mut wallets: Vec<String> = Vec::new();
    let mut next_uri = format!(
        "https://api.dune.com/api/v1/query/3585761/results?limit={}",
        env::var("DUNE_LINES_PER_REQUEST")?
    );

    loop {
        let result = client
            .get(&next_uri)
            .header("X-Dune-API-Key", env::var("DUNE_API_KEY")?)
            .send()
            .await?
            .json::<DuneResponse>()
            .await?;
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
    }

    Ok(wallets
        .into_iter()
        .collect::<std::collections::HashSet<String>>()
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect())
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserResponse {
    referral_code: String,
    signed: bool,
    signed_build_and_earn: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PointsResponse {
    total_points: String,
    total_ref_points: String,
    total_build_points: String,
    total_extra_points: String,
    total_pendle_points: String,
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
