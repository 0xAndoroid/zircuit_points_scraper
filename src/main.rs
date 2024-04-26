use std::env;
use std::time::Duration;

use fancy_duration::AsFancyDuration;
use file::write_points;
use log::{error, info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};

use crate::dune::fetch_users;
use crate::file::{read_points, read_wallets, write_wallets};

mod file;
mod dune;

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
            let users = match fetch_users(&client).await {
                Ok(users) => users,
                Err(e) => {
                    error!("Error fetching users: {:?}", e);
                    return;
                }
            };
            write_wallets(&users).unwrap();
            users
        }
    };
    let users = users
        .into_iter()
        .filter(|u| {
            u != "0x7a493be5c2ce014cd049bf178a1ac0db1b434744" // SY Pendle
                && u != "0x34349c5569e7b846c3558961552d2202760a9789" // SY Pendle
                && u != "0xd7df7e085214743530aff339afc420c7c720bfa7" // SY Pendle
                && u != "0x0000000000000000000000000000000000000000" // Zero address
                && u != "0x52Aa899454998Be5b000Ad077a46Bbe360F4e497" // Fluid
        })
        .collect::<Vec<String>>();
    let mut fetched_users = 0;
    let timer = std::time::Instant::now();
    info!("Total users: {}", users.len());

    let mut user_infos = match read_points() {
        Ok(users) => users,
        Err(_) => Vec::new(),
    };
    if users.len() == user_infos.len() {
        info!("All users have been prefetched!");
        return;
    }
    if user_infos.len() % 250 != 0 {
        let last_chunk = user_infos.len() % 250;
        user_infos.truncate(user_infos.len() - last_chunk);
    }
    info!("Prefetched users: {}", user_infos.len());
    let total_users = users.len() - user_infos.len();
    info!("To be fetched users: {}", total_users);

    let fetch_referral_codes =
        env::var("FETCH_REFERRAL_CODES").unwrap_or("false".to_string()).parse::<bool>().unwrap();

    // Increasing chunk size causes rate limiting error
    let chunk_size =
        env::var("ZIRCUIT_BATCH_SIZE").unwrap_or("25".to_string()).parse::<usize>().unwrap();

    let mut skip_chunks = user_infos.len() / chunk_size;

    for users_chunk in users.chunks(chunk_size) {
        if skip_chunks > 0 {
            skip_chunks -= 1;
            continue;
        }
        let mut handles = Vec::new();
        for user in users_chunk {
            let client = client.clone();
            let user_cl = user.clone();
            let handle = tokio::spawn(async move {
                fetch_user_info(&client, &user_cl, fetch_referral_codes).await
            });
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
                .unwrap() ;
            if user_info.is_err() {
                error!("Error fetching user info {}: {:?}", user, user_info.err().unwrap());
                continue;
            }
            user_infos.push(user_info.unwrap());
            fetched_users += 1;
            if fetched_users % 250 == 0 {
                info!("Fetched {}/{}", fetched_users, total_users);
                let ellapsed = Duration::from_secs(timer.elapsed().as_secs());
                let remaining = Duration::from_secs(
                    (ellapsed * (total_users as u32 - fetched_users as u32) / fetched_users as u32)
                        .as_secs(),
                );
                info!(
                    "Elapsed time: {} / {}",
                    ellapsed.fancy_duration().to_string(),
                    remaining.fancy_duration().to_string()
                );
                write_points(&user_infos).unwrap();
            }
        }
    }
    info!("Finished fetching all users!");
    info!("Elapsed time: {:?}", timer.elapsed());
    write_points(&user_infos).unwrap();
}

async fn fetch_user_info(
    client: &Client,
    address: &str,
    fetch_referral_codes: bool,
) -> Result<User, anyhow::Error> {
    let user_response = if fetch_referral_codes {
        client
            .get(format!("https://stake.zircuit.com/api/user/{}", address))
            .send()
            .await?
            .json::<UserResponse>()
            .await
    } else {
        Ok(UserResponse::default())
    };
    let points_response = client
        .get(format!("https://stake.zircuit.com/api/points/{}", address))
        .send()
        .await?
        .text()
        .await;

    let user: UserResponse = match user_response {
        Ok(user) => user,
        _ => UserResponse::default(),
    };

    let points: PointsResponse = match points_response {
        Ok(points) => match serde_json::from_str(&points) {
            Ok(points) => points,
            Err(_) => {
                error!("Failed fetching points of user {}", address);
                error!("{}", points);
                PointsResponse::default()
            }
        }
        _ => PointsResponse::default(),
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

#[derive(Deserialize, Serialize, Default)]
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

impl Default for PointsResponse {
    fn default() -> Self {
        PointsResponse {
            total_points: "0".to_string(),
            total_ref_points: "0".to_string(),
            total_build_points: "0".to_string(),
            total_extra_points: "0".to_string(),
            total_pendle_points: "0".to_string(),
        }
    }
}

