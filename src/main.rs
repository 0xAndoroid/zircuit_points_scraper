use std::env;

use log::{error, info};
use reqwest::Client;
use serde::{Deserialize, Serialize};

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    env_logger::init();
    let client = Client::new();
    let users = fetch_users(&client).await.unwrap();
    let total_users = users.len();
    let mut fetched_users = 0;
    let timer = std::time::Instant::now();
    info!("Total users: {}", total_users);

    let mut user_infos = Vec::new();

    for user in users {
        let user_info = fetch_user_info(&client, &user)
            .await
            .map_err(|e| {
                error!("Error fetching user {}: {}", &user, e);
                e
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
    info!("Finished fetching all users!");
    info!("Elapsed time: {:?}", timer.elapsed());
    write_csv(&user_infos).unwrap();
}

fn write_csv(users: &Vec<User>) -> Result<(), anyhow::Error> {
    let mut wtr = csv::Writer::from_path("users.csv")?;
    for user in users {
        wtr.serialize(user)?;
    }
    wtr.flush()?;
    Ok(())
}

async fn fetch_user_info(client: &Client, address: &str) -> Result<User, anyhow::Error> {
    let user_response = client
        .get(format!("https://stake.zircuit.com/api/user/{}", address))
        .send()
        .await?
        .json::<UserResponse>()
        .await?;
    let points_response = client
        .get(format!("https://stake.zircuit.com/api/points/{}", address))
        .send()
        .await?
        .json::<PointsResponse>()
        .await?;

    let user: UserResponseFull = match user_response {
        UserResponse::Full(user) => user,
        UserResponse::Error { message: _ } => UserResponseFull {
            referral_code: "".to_string(),
            signed: false,
            signed_build_and_earn: false,
        },
    };

    let points: PointsResponseFull = match points_response {
        PointsResponse::Full(points) => points,
        PointsResponse::Empty(_) => PointsResponseFull {
            total_points: "0".to_string(),
            total_ref_points: "0".to_string(),
            total_build_points: "0".to_string(),
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
}

async fn fetch_users(client: &Client) -> Result<Vec<String>, anyhow::Error> {
    Ok(client
        .get("https://api.dune.com/api/v1/query/3459485/results?limit=100000")
        .header("X-Dune-API-Key", env::var("DUNE_API_KEY")?)
        .send()
        .await?
        .json::<DuneResponse>()
        .await?
        .result
        .rows
        .iter()
        .map(|row| match &row.from {
            Some(from) => from.to_string(),
            None => "".to_string(),
        })
        .collect::<Vec<String>>()
        .into_iter()
        .collect::<std::collections::HashSet<String>>()
        .into_iter()
        .filter(|s| !s.is_empty())
        .collect::<Vec<String>>())
}

#[derive(Deserialize)]
#[serde(untagged)]
enum UserResponse {
    Full(UserResponseFull),
    Error {
        #[allow(dead_code)]
        message: String,
    },
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct UserResponseFull {
    referral_code: String,
    signed: bool,
    signed_build_and_earn: bool,
}

#[derive(Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
struct PointsResponseFull {
    total_points: String,
    total_ref_points: String,
    total_build_points: String,
}

#[derive(Deserialize, Serialize)]
#[serde(untagged)]
enum PointsResponse {
    Full(PointsResponseFull),
    Empty(Vec<String>),
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
    // next_uri: String,
    // next_offset: u64,
}

#[derive(Deserialize, Serialize)]
struct DuneResult {
    rows: Vec<Rows>,
    metadata: DuneMetadata,
}

#[derive(Deserialize, Serialize)]
struct Rows {
    evt_block_time: String,
    evt_block_number: u64,
    from: Option<String>,
    to: String,
    evt_tx_hash: String,
    #[serde(rename = "type")]
    type_of_transfer: String,
    symbol: String,
    transfers: f64,
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
