use crate::fetch::{User, UserSeason1, UserSeason2};

pub fn write_points(users: &Vec<User>) -> Result<(), anyhow::Error> {
    let mut wtr_1 = csv::Writer::from_path("users_season1.csv")?;
    let mut wtr_2 = csv::Writer::from_path("users_season2.csv")?;
    for user in users {
        wtr_1.serialize(user.season1.clone())?;
        wtr_2.serialize(user.season2.clone())?;
    }
    wtr_1.flush()?;
    wtr_2.flush()?;
    Ok(())
}

pub fn write_wallets(wallets: &Vec<String>) -> Result<(), anyhow::Error> {
    let mut wtr = csv::Writer::from_path("wallets.csv")?;
    for wallet in wallets {
        wtr.serialize(wallet)?;
    }
    wtr.flush()?;
    Ok(())
}

pub fn read_points() -> Result<Vec<User>, anyhow::Error> {
    let rdr_s1 = csv::Reader::from_path("users_season1.csv")?;
    let rdr_s2 = csv::Reader::from_path("users_season2.csv")?;
    rdr_s1
        .into_deserialize::<UserSeason1>()
        .zip(rdr_s2.into_deserialize::<UserSeason2>())
        .map(|(s1, s2)| -> Result<User, anyhow::Error> { Ok(User::from((s1?, s2?))) })
        .collect::<Result<Vec<User>, anyhow::Error>>()
}

pub fn read_wallets() -> Result<Vec<String>, anyhow::Error> {
    let mut rdr = csv::Reader::from_path("wallets.csv")?;
    rdr.set_headers(csv::StringRecord::from(vec!["wallet"]));
    let mut wallets = Vec::new();
    for result in rdr.deserialize() {
        let wallet: String = result?;
        wallets.push(wallet);
    }
    Ok(wallets)
}
