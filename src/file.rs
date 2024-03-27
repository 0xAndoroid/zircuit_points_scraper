use crate::User;

pub fn write_csv(users: &Vec<User>) -> Result<(), anyhow::Error> {
    let mut wtr = csv::Writer::from_path("users.csv")?;
    for user in users {
        wtr.serialize(user)?;
    }
    wtr.flush()?;
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

pub fn read_wallets() -> Result<Vec<String>, anyhow::Error> {
    let mut rdr = csv::Reader::from_path("wallets.csv")?;
    let mut wallets = Vec::new();
    for result in rdr.deserialize() {
        let wallet: String = result?;
        wallets.push(wallet);
    }
    Ok(wallets)
}
