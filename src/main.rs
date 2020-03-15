#[macro_use]
extern crate lazy_static;

use hex_string::HexString;
use hmac::Mac;
use serde_derive::{Deserialize, Serialize};

mod filters;
mod handlers;
mod models;
mod teams_db;
mod scoreboard;

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


use crate::models::TeamName;
use teams_db::TeamsDb;

lazy_static!{
    pub static ref SECRET_KEY: [u8; 32] = {
        rand::random()
    };
}

fn sign_on_team_name(team_name: &TeamName) -> TeamToken {
    let mut mac = hmac::Hmac::<sha2::Sha256>::new_varkey(&*SECRET_KEY)
        .expect("Hmac init should never be a problem");

    mac.input(team_name.as_str().as_bytes());

    TeamToken {
        token: mac.result().code().as_slice().into(),
    }
}

fn verify_team_token(token: &TeamToken, team_name: &TeamName) -> bool {
    let mut mac = hmac::Hmac::<sha2::Sha256>::new_varkey(&*SECRET_KEY)
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

    pretty_env_logger::init();

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
    use crate::models::solution::{Solution, ChallengeDate};
    use std::collections::HashMap;
    use crate::models::TeamName;
    use crate::scoreboard::Score;

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

        let api = crate::filters::game_api(teams_db.clone(), scoreboard.clone());

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

        let solution = Solution {
            challenge: ChallengeDate::Qualification(2020),
            solutions: {
                let mut h = HashMap::new();
                h.insert("a".into(),
                         include_str!("../../hashcode_score_calc/assets/2020qual/submissions/example_submission.txt").to_owned());
                h
            }
        };

        let res = warp::test::request()
            .method("POST")
            .path(&submit_path)
            .json(&solution)
            .reply(&api)
            .await;

        assert_eq!(
            res.status(),
            http::StatusCode::OK,
            "failed to submit as team in {} with body {:?}",
            submit_path,
            res.body()
        );
        assert_eq!(res.body(), "16");

        assert_eq!(scoreboard.get_best_score(&new_team.name).await,
        Some(16),
        "The example should score 16 points");
    }

    #[tokio::test]
    async fn test_scoreboard() {
        use hex_string::HexString;
        use crate::scoreboard::ScoreBoard;
        use crate::models::solution::ChallengeDate;

        let teams_db = TeamsDb::new();
        let scoreboard = ScoreBoard::new();

        let api = crate::filters::game_api(teams_db.clone(), scoreboard.clone());

        let empty_scoreboard = {
            let res = warp::test::request()
                .path("/scoreboard")
                .method("GET")
                .reply(&api)
                .await;

            assert_eq!(
                res.status(),
                http::StatusCode::OK,
                "Couldn't retrieve scoreboard"
            );

            serde_json::from_slice::<HashMap<TeamName, Score>>(res.body()).expect("Should be a json")
        };

        assert_eq!(
            empty_scoreboard.len(),
            0,
            "scoreboard should have been empty"
        );

        

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

        let solution = Solution {
            challenge: ChallengeDate::Qualification(2020),
            solutions: {
                let mut h = HashMap::new();
                h.insert("a".into(),
                         include_str!("../../hashcode_score_calc/assets/2020qual/submissions/example_submission.txt").to_owned());
                h
            }
        };

        let res = warp::test::request()
            .method("POST")
            .path(&submit_path)
            .json(&solution)
            .reply(&api)
            .await;

        assert_eq!(
            res.status(),
            http::StatusCode::OK,
            "failed to submit as team in {} with body {:?}",
            submit_path,
            res.body()
        );
        assert_eq!(res.body(), "16");

        let score = {
            let res = warp::test::request()
                .path("/scoreboard")
                .method("GET")
                .reply(&api)
                .await;

            assert_eq!(
                res.status(),
                http::StatusCode::OK,
                "Couldn't retrieve scoreboard"
            );

            serde_json::from_slice::<HashMap<TeamName, Score>>(res.body()).expect("Should be a json")
        };

        assert_eq!(score[&new_team.name], 16);

    }
}
