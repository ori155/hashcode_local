use hex_string::HexString;
use hmac::Mac;
use serde_derive::{Deserialize, Serialize};

mod filters;
mod handlers;
mod models;
mod teams_db;

use models::team::Team;

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

mod scoreboard {
    use std::sync::Arc;
    use tokio::sync::RwLock;
    use std::collections::HashMap;
    use crate::models::TeamName;

    type Score = u64;

    #[derive(Clone)]
    pub struct ScoreBoard {
    db: Arc<RwLock<HashMap<TeamName, Vec<Score>>>>
}

    impl ScoreBoard {
        pub fn new() -> Self {
            Self { db: Arc::new(RwLock::new(HashMap::new())) }
        }

        pub async fn add_team_score(&mut self, team_name: &TeamName, score: Score) {
            self.db.write().await
                .entry(team_name.clone())
                .or_default()
                .push(score);
        }

        pub async fn get_best_score(&self, team_name: &TeamName) -> Option<Score> {
            self.db.read().await
                .get(team_name)
                .map(|score_vec| score_vec.iter().max().clone())
                .flatten()
                .map(|s| *s)
        }
    }

    #[cfg(test)]
    mod tests {
        use crate::scoreboard::ScoreBoard;
        use crate::models::TeamName;

        #[tokio::test]
        async fn can_add_team() {

            let team = TeamName::from("abc");

            let mut score_board = ScoreBoard::new();
            score_board.add_team_score(&team, 120).await;

        }

        #[tokio::test]
        async fn can_get_best_score() {

            let team = TeamName::from("abc");

            let mut score_board = ScoreBoard::new();
            score_board.add_team_score(&team, 120).await;

            assert_eq!(
                score_board.get_best_score(&team).await,
                Some(120)
            )
        }
    }
}

use crate::models::TeamName;
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
    use scoreboard::ScoreBoard;

    let teams = TeamsDb::new();
    let scoreboard = ScoreBoard::new();

    warp::serve(game_api(teams, scoreboard))
        .run(([127, 0, 0, 1], 8080))
        .await;
}

#[cfg(test)]
mod tests {
    use crate::teams_db::TeamsDb;
    use crate::{Team, TeamToken};

    #[tokio::test]
    async fn test_list_empty_teams() {
        use crate::scoreboard::ScoreBoard;
        let teams_db = TeamsDb::new();
        let scoreboard = ScoreBoard::new();
        let api = crate::filters::game_api(teams_db.clone(), scoreboard);

        let res = warp::test::request().path("/teams").reply(&api).await;

        assert_eq!(res.status(), http::StatusCode::OK);
        assert_eq!(res.body(), "[]");
    }

    #[tokio::test]
    async fn test_add_team() {
        use crate::scoreboard::ScoreBoard;

        let teams_db = TeamsDb::new();
        let scoreboard = ScoreBoard::new();
        let api = crate::filters::game_api(teams_db.clone(), scoreboard);

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
        use crate::scoreboard::ScoreBoard;

        let teams_db = TeamsDb::new();
        let scoreboard = ScoreBoard::new();

        let api = crate::filters::game_api(teams_db.clone(), scoreboard);

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
