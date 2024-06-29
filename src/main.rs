use std::env;
use std::sync::atomic::AtomicUsize;
use std::sync::Arc;

use file::write_points;
use indicatif::{ProgressBar, ProgressStyle};
use log::{error, info, warn};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use tokio::sync::mpsc;

use crate::dune::fetch_users;
use crate::file::{read_points, read_wallets, write_wallets};

mod dune;
mod file;

const PROXY: &str = include_str!("../proxies.json");

#[tokio::main]
async fn main() {
    dotenv::dotenv().ok();
    if env::var("RUST_LOG").is_err() {
        env::set_var("RUST_LOG", "info");
    }
    env_logger::init();
    info!("Starting Zircuit wallet fetcher");
    let clients = init_clients().await;
    info!("Clients initialized: {}", clients.len());
    let users = match read_wallets() {
        Ok(users) => users,
        Err(_) => {
            warn!("No wallets found, fetching from Dune API");
            let users = match fetch_users().await {
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
    let mut users = users
        .into_iter()
        .filter(|u| {
            let u = u.to_lowercase();
            u != "0x7a493be5c2ce014cd049bf178a1ac0db1b434744".to_lowercase() // SY Pendle
                && u != "0x34349c5569e7b846c3558961552d2202760a9789".to_lowercase() // SY Pendle
                && u != "0xd7df7e085214743530aff339afc420c7c720bfa7".to_lowercase() // SY Pendle
                && u != "0x293C6937D8D82e05B01335F7B33FBA0c8e256E30".to_lowercase() // SY Pendle
                && u != "0x0000000000000000000000000000000000000000".to_lowercase() // Zero address
                && u != "0x52Aa899454998Be5b000Ad077a46Bbe360F4e497".to_lowercase()
            // Fluid
        })
        .collect::<Vec<String>>();
    info!("Total users: {}", users.len());

    let mut user_infos = match read_points() {
        Ok(users) => users,
        Err(_) => Vec::new(),
    };

    if users.len() == user_infos.len() {
        info!("All users have been prefetched!");
        return;
    }
    
    info!("Prefetched users: {}", user_infos.len());

    users.retain(|u| !user_infos.iter().any(|ui| ui.address == *u));
    let fetched_users: Arc<AtomicUsize> = Arc::new(0.into());
    let total_users = users.len();
    info!("To be fetched users: {}", total_users);
    let users = Arc::new(users);
    let clients_len = clients.len();

    #[allow(clippy::let_underscore_future)]
    let _ = tokio::spawn(progress_bar(fetched_users.clone(), total_users));

    let (tx, mut rx) = mpsc::channel::<User>(250);

    let mut handles = Vec::new();
    for client in clients {
        let users = users.clone();
        let fetched_users = fetched_users.clone();
        let tx = tx.clone();
        let handle = tokio::spawn(async move {
            run_one_client(client, users, fetched_users, tx).await;
        });
        handles.push(handle);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }

    while let (Some(user), f) = (rx.recv().await, fetched_users.load(std::sync::atomic::Ordering::SeqCst)) {
        if f == users.len() + clients_len {
            break;
        }
        user_infos.push(user);
        if f % 250 == 0 {
            write_points(&user_infos).unwrap();
        }
    }

    write_points(&user_infos).unwrap();
}

async fn progress_bar(fetched_users: Arc<AtomicUsize>, total_users: usize) {
    let progress_bar = ProgressBar::new(total_users as u64);
    progress_bar.set_style(
        ProgressStyle::with_template(
            "{spinner:.green} [{elapsed_precise}] [{wide_bar:.cyan/blue}] {pos:>7}/{len:7} ({eta})",
        )
        .unwrap()
        .progress_chars("#>-"),
    );
    progress_bar.set_position(0);

    loop {
        let f = fetched_users.load(std::sync::atomic::Ordering::SeqCst);
        progress_bar.set_position(f as u64);
        tokio::time::sleep(tokio::time::Duration::from_millis(100)).await;
    }
}

async fn run_one_client(
    client: Client,
    users: Arc<Vec<String>>,
    fetched_users: Arc<AtomicUsize>,
    mpsc: mpsc::Sender<User>,
) {
    loop {
        let user_to_fetch = fetched_users.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
        if user_to_fetch >= users.len() {
            break;
        }
        let sleep_task = tokio::time::sleep(tokio::time::Duration::from_millis(
            std::env::var("ZIRCUIT_COOLDOWN").unwrap_or("1100".to_string()).parse::<u64>().unwrap(),
        ));
        let user_addr = users[user_to_fetch].clone();
        let user = tryhard::retry_fn(|| async {
            fetch_user_info(&client, &user_addr).await
        })
        .retries(5)
        .exponential_backoff(std::time::Duration::from_secs(5))
        .max_delay(std::time::Duration::from_secs(300))
        .await;

        if user.is_err() {
            error!("Error fetching user info {}: {:?}", user_addr, user.err().unwrap());
            continue;
        }

        mpsc.send(user.unwrap()).await.unwrap();

        sleep_task.await;
    }
}

// async fn verify_ips_are_different(clients: &Vec<Client>) {
//     let mut hashset = std::collections::HashSet::new();
//     let mut handles = Vec::new();
//     for client in clients {
//         let client = client.clone();
//         let ip_h = tokio::spawn(async move {
//             client.get("https://api.ipify.org").send().await.unwrap().text().await.unwrap()
//         });
//         handles.push(ip_h);
//     }
//
//     for handle in handles {
//         let ip = handle.await.unwrap();
//         hashset.insert(ip.clone());
//     }
//
//     if hashset.len() != clients.len() {
//         error!("IPs are not different!");
//         std::process::exit(1);
//     }
// }

async fn init_clients() -> Vec<Client> {
    let proxies: Vec<String> = match serde_json::from_str(PROXY) {
        Ok(proxies) => proxies,
        Err(_) => {
            warn!("No proxies found, will use direct connection");
            Vec::new()
        }
    };
    
    let mut clients = Vec::new();

    clients.push(Client::new());

    for proxy in proxies {
        let proxy = reqwest::Proxy::all(proxy).unwrap();
        let client = Client::builder().proxy(proxy).build().unwrap();
        clients.push(client);
    }
    clients
}

async fn fetch_user_info(
    client: &Client,
    address: &str,
) -> Result<User, anyhow::Error> {
    let points_response = client
        .get(format!("https://stake.zircuit.com/api/points/{}", address))
        .send()
        .await?
        .text()
        .await;

    let points: PointsResponse = match points_response {
        Ok(points) => match serde_json::from_str(&points) {
            Ok(points) => points,
            Err(_) => {
                anyhow::bail!("{}", points);
            }
        },
        _ => anyhow::bail!("Unknown error"),
    };

    Ok(User {
        address: address.to_string(),
        referral_code: "".to_string(),
        signed: false,
        signed_build_and_earn: false,
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
