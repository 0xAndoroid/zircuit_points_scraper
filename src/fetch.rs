use reqwest::Client;
use serde::{Deserialize, Serialize};

pub async fn fetch_user_info(client: &Client, address: &str) -> Result<User, anyhow::Error> {
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

    User::new(address, points)
}

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct User {
    pub address: String,
    pub season1: UserSeason1,
    pub season2: UserSeason2,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct UserSeason1 {
    pub address: String,

    pub total_points: f64,
    pub total_staking_points: f64,
    pub total_ref_points: f64,
    pub total_pendle_points: f64,
    pub total_build_points: f64,
}

#[derive(Deserialize, Serialize, Debug, Clone, PartialEq)]
pub struct UserSeason2 {
    pub address: String,

    pub total_points: f64,
    pub total_staking_points: f64,
    pub total_ref_points: f64,
    pub total_build_points: f64,
    pub total_ref_build_points: f64,
    pub total_pendle_points: f64,
}

impl User {
    fn new(address: &str, pts: PointsResponse) -> Result<Self, anyhow::Error> {
        let season1 = UserSeason1 {
            address: address.to_string(),
            total_points: f64::try_from(&pts.season1_points.total_points)?,
            total_staking_points: f64::try_from(&pts.season1_points.total_staking_points)?,
            total_ref_points: f64::try_from(&pts.season1_points.total_ref_points)?,
            total_pendle_points: f64::try_from(&pts.season1_points.total_pendle_points)?,
            total_build_points: f64::try_from(&pts.season1_points.total_build_points)?,
        };

        let season2 = UserSeason2 {
            address: address.to_string(),
            total_points: f64::try_from(&pts.total_points)?,
            total_staking_points: f64::try_from(&pts.total_staking_points)?,
            total_ref_points: f64::try_from(&pts.total_ref_points)?,
            total_build_points: f64::try_from(&pts.total_build_points)?,
            total_ref_build_points: f64::try_from(&pts.total_ref_build_points)?,
            total_pendle_points: f64::try_from(&pts.total_pendle_points)?,
        };

        Ok(User {
            address: address.to_string(),
            season1,
            season2,
        })
    }
}

impl From<(UserSeason1, UserSeason2)> for User {
    fn from((s1, s2): (UserSeason1, UserSeason2)) -> Self {
        assert_eq!(s1.address, s2.address);
        User {
            address: s1.address.clone(),
            season1: s1,
            season2: s2,
        }
    }
}

#[derive(Deserialize, Serialize, Default)]
#[serde(default)] // Other fields shouldn't be missing, but just in case
#[serde(rename_all = "camelCase")]
struct PointsResponse {
    total_points: Value,
    #[serde(default)] // Missing if user isn't a season1 participant
    season1_points: Season1PointsResponse,
    total_staking_points: Value,
    total_ref_points: Value,
    total_build_points: Value,
    total_ref_build_points: Value,
    #[serde(default)] // If user isn't a pendle user, this field might be missing
    total_pendle_points: Value,
    total_instadapp_points: Value,
    #[serde(rename = "totalOKXPoints")]
    total_okx_points: Value,
    total_extra_points: Value,
    is_pendle_user: bool,
}

#[derive(Deserialize, Serialize, Default)]
#[serde(default)] // Other fields shouldn't be missing, but just in case
#[serde(rename_all = "camelCase")]
struct Season1PointsResponse {
    total_points: Value,
    total_staking_points: Value,
    total_ref_points: Value,
    #[serde(default)] // If user isn't a pendle user, this field might be missing
    total_pendle_points: Value,
    total_build_points: Value,
}

#[derive(Deserialize, Serialize, Debug, PartialEq)]
#[serde(untagged)]
enum Value {
    String(String),
    Float(f64),
}

impl Default for Value {
    fn default() -> Self {
        Value::Float(0.0)
    }
}

impl TryFrom<&Value> for f64 {
    type Error = anyhow::Error;

    fn try_from(value: &Value) -> Result<f64, Self::Error> {
        Ok(match value {
            Value::String(s) => s.parse()?,
            Value::Float(f) => *f,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn deserialize_points_response() {
        let s = r#"{"totalPoints":"0","season1Points":{"totalPoints":535.5548897963093,"totalStakingPoints":535.5548897963093,"totalRefPoints":0,"totalBuildPoints":0},"totalStakingPoints":"0","totalRefPoints":"0","totalBuildPoints":"0","totalRefBuildPoints":"0","totalPendlePoints":"0","totalInstadappPoints":"0","totalOKXPoints":"0","totalExtraPoints":"0","isPendleUser":false}"#;
        let points: PointsResponse = serde_json::from_str(s).unwrap();
        let user = User::new("0x123", points).unwrap();
        let expected_user = User {
            address: "0x123".to_string(),
            season1: UserSeason1 {
                address: "0x123".to_string(),
                total_points: 535.5548897963093,
                total_staking_points: 535.5548897963093,
                total_ref_points: 0.0,
                total_pendle_points: 0.0,
                total_build_points: 0.0,
            },
            season2: UserSeason2 {
                address: "0x123".to_string(),
                total_points: 0.0,
                total_staking_points: 0.0,
                total_ref_points: 0.0,
                total_build_points: 0.0,
                total_ref_build_points: 0.0,
                total_pendle_points: 0.0,
            },
        };
        assert_eq!(user, expected_user);
    }
}
