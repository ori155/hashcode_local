use hex_string::HexString;
use hmac::Mac;
use serde_derive::{Deserialize, Serialize};

mod filters;
mod handlers;
pub mod team;
pub mod teams_db;
pub use team::Team;

#[derive(Debug, Serialize, Deserialize)]
enum ApiError {
    ErrorTeamExists,
    WrongToken,
}
impl warp::reject::Reject for ApiError {}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamToken {
    pub token: Vec<u8>,
}

impl std::convert::From<HexString> for TeamToken {
    fn from(hs: HexString) -> Self {
        Self {
            token: hs.as_bytes(),
        }
    }
}

pub struct AccessGranted {
    pub team: TeamName,
}

use crate::team::TeamName;
use teams_db::TeamsDb;

//TODO: make secret key be randomized each run
const SECRET_KEY: &[u8] = b"This is my secret key";

fn sign_on_team_name(team_name: &TeamName) -> TeamToken {
    let mut mac = hmac::Hmac::<sha2::Sha256>::new_varkey(&SECRET_KEY)
        .expect("Hmac init should never be a problem");

    mac.input(team_name.as_str().as_bytes());

    TeamToken {
        token: mac.result().code().as_slice().into(),
    }
}

fn verify_team_token(token: &TeamToken, team_name: &TeamName) -> bool {
    let mut mac = hmac::Hmac::<sha2::Sha256>::new_varkey(&SECRET_KEY)
        .expect("Hmac init should never be a problem");

    mac.input(team_name.as_str().as_bytes());

    match mac.verify(&token.token) {
        Ok(_) => true,
        Err(_) => false,
    }
}

#[tokio::main]
async fn main() {
    use filters::game_api;

    let teams = TeamsDb::new();

    warp::serve(game_api(teams))
        .run(([127, 0, 0, 1], 8080))
        .await;
}

#[cfg(test)]
mod tests {
    use crate::teams_db::TeamsDb;
    use crate::{Team, TeamToken};

    #[tokio::test]
    async fn test_list_empty_teams() {
        let teams_db = TeamsDb::new();
        let api = crate::filters::game_api(teams_db.clone());

        let res = warp::test::request().path("/teams").reply(&api).await;

        assert_eq!(res.status(), http::StatusCode::OK);
        assert_eq!(res.body(), "[]");
    }

    #[tokio::test]
    async fn test_add_team() {
        let teams_db = TeamsDb::new();
        let api = crate::filters::game_api(teams_db.clone());

        let new_team = Team {
            name: "first_team".into(),
            participants: vec!["ori".to_owned()],
        };

        let res = warp::test::request()
            .path("/register_team")
            .method("POST")
            .json(&new_team)
            .reply(&api)
            .await;

        assert_eq!(res.status(), http::StatusCode::OK);

        assert!(teams_db.contains(&new_team.name).await);
    }

    #[tokio::test]
    async fn test_team_access() {
        use hex_string::HexString;
        let teams_db = TeamsDb::new();
        let api = crate::filters::game_api(teams_db.clone());

        let new_team = Team {
            name: "first_team".into(),
            participants: vec!["ori".to_owned()],
        };

        let res = warp::test::request()
            .path("/register_team")
            .method("POST")
            .json(&new_team)
            .reply(&api)
            .await;

        assert_eq!(
            res.status(),
            http::StatusCode::OK,
            "team registration failed"
        );
        let team_token: TeamToken =
            serde_json::from_slice(res.body()).expect("should receive token");

        let submit_path = format!(
            "/team/{}/{}/submit",
            new_team.name,
            HexString::from_bytes(&team_token.token).as_str()
        );
        let res = warp::test::request()
            .method("POST")
            .path(&submit_path)
            .reply(&api)
            .await;

        assert_eq!(
            res.status(),
            http::StatusCode::OK,
            "failed to submit as team in {} with body {:?}",
            submit_path,
            res.body()
        );
        assert_eq!(res.body(), "");
    }
}
