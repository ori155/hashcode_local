use warp::Filter;

use serde_derive::{Serialize, Deserialize};
use hex_string::HexString;
use hmac::Mac;

pub mod team;
pub use team::Team;

#[derive(Debug, Serialize, Deserialize)]
enum ApiError{
    ErrorTeamExists,
    WrongToken
}
impl warp::reject::Reject for ApiError {}

#[derive(Debug, Serialize, Deserialize)]
pub struct TeamToken {
    pub token: Vec<u8>
}

impl std::convert::From<HexString> for TeamToken {
    fn from(hs: HexString) -> Self {
        Self{token: hs.as_bytes()}
    }
}

pub struct AccessGranted {
    pub team: TeamName
}

pub mod teams_db {
    use tokio::sync::RwLock;
    use std::sync::Arc;
    use std::collections::HashMap;

    use crate::team::{Team, TeamName};

    #[derive(Clone)]
    pub struct TeamsDb {
        inner: Arc<RwLock<HashMap<TeamName, Team>>>
    }

    impl TeamsDb {
        pub fn new() -> Self {
            Self{inner: Arc::new(RwLock::new(HashMap::new()))}
        }

        pub async fn contains(&self, team_name: &TeamName) -> bool {
            self.inner.read().await
                .contains_key(team_name)
        }

        pub async fn insert(&mut self, team: Team) {
            let key = team.name.clone();
            self.inner.write().await
                .insert(key, team);
        }

        //TODO change to iterator
        pub async fn list_team_names(&self) -> Vec<TeamName> {
            self.inner.read().await
                .keys().map(|s| s.clone()).collect()
        }
    }

}

use teams_db::TeamsDb;
use crate::filters::team_registration;
use crate::team::TeamName;


mod handlers {
    use super::{Team, TeamsDb};
    use crate::team::TeamName;
    use crate::{sign_on_team_name, TeamToken, AccessGranted};
    use hex_string::HexString;

    pub async fn add_team(new_team: Team, mut teams_db: TeamsDb) -> Result<impl warp::Reply, std::convert::Infallible> {

        if teams_db.contains(&new_team.name).await {
            return Ok(warp::reply::json(&crate::ApiError::ErrorTeamExists))
        }

        let new_team_token = sign_on_team_name(&new_team.name);
        teams_db.insert(new_team).await;

        Ok(warp::reply::json(&new_team_token))
    }

    pub async fn list_teams(teams_db: TeamsDb) -> Result<impl warp::Reply, std::convert::Infallible> {
        let listed_teams: Vec<TeamName> = teams_db.list_team_names().await;

        Ok(warp::reply::json(&listed_teams))
    }

    pub async fn test_team_token(team_name: TeamName, team_token: HexString) -> Result<AccessGranted, warp::Rejection> {
        let team_token: TeamToken = team_token.into();

        if crate::verify_team_token(&team_token, &team_name) {
            Ok(AccessGranted{ team: team_name})
        } else {
            Err(warp::reject::custom(crate::ApiError::WrongToken))
        }

    }
}

//TODO: make secret key be randomized each run
const SECRET_KEY: &[u8] = b"This is my secret key";

fn sign_on_team_name(team_name: &TeamName) -> TeamToken {
    println!("sign on team name");
    let mut mac = hmac::Hmac::<sha2::Sha256>::new_varkey(&SECRET_KEY)
        .expect("Hmac init should never be a problem");

    mac.input(team_name.as_str().as_bytes());

    TeamToken { token: mac.result().code().as_slice().into() }
}

fn verify_team_token(token: &TeamToken, team_name: &TeamName) -> bool {
    println!("verify team");
    let mut mac = hmac::Hmac::<sha2::Sha256>::new_varkey(&SECRET_KEY)
        .expect("Hmac init should never be a problem");

    mac.input(team_name.as_str().as_bytes());

    match mac.verify(&token.token) {
        Ok(_) => true,
        Err(_) => false
    }
}

mod filters {
    use crate::teams_db::TeamsDb;
    use warp::Filter;
    use crate::team::TeamName;
    use hex_string::HexString;
    use crate::AccessGranted;

    fn with_db(db: TeamsDb) -> impl Filter<Extract = (TeamsDb,), Error = std::convert::Infallible> + Clone {
        warp::any().map(move || db.clone())
    }

    pub fn team_registration(teams: TeamsDb) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        warp::post()
            .and(warp::path::path("register_team"))
            .and(warp::body::json())
            .and(with_db(teams))
            .and_then( crate::handlers::add_team)
    }

    pub fn list_teams(teams: TeamsDb) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        warp::get()
            .and(warp::path("teams"))
            .and(warp::path::end())
            .and(with_db(teams.clone()))
            .and_then(crate::handlers::list_teams)
    }

    pub fn team_access(teams: TeamsDb) -> impl Filter<Extract = (AccessGranted,), Error = warp::Rejection> + Clone
    {
        //Todo: test team exists
        warp::path!("team" / TeamName / HexString / ..)
            .and_then(crate::handlers::test_team_token)
    }

    pub fn submit_solution(teams: TeamsDb) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        warp::post()
            .and(team_access(teams.clone()))
            .and(warp::path::path("submit"))
            .map(|team_accessed: AccessGranted| {
                format!("Legit submission to {}", team_accessed.team)
            })
    }

    pub fn game_api(teams: TeamsDb) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        team_registration(teams.clone())
            .or(list_teams(teams.clone()))
            .or(submit_solution(teams.clone()))
    }
}

#[tokio::main]
async fn main() {
    use filters::game_api;

    let teams = TeamsDb::new();

    warp::serve(game_api(teams)).run(([127, 0, 0, 1], 8080)).await;

}

#[cfg(test)]
mod tests {
    use crate::teams_db::TeamsDb;
    use crate::{Team, TeamToken};

    #[tokio::test]
    async fn test_list_empty_teams() {
        let teams_db = TeamsDb::new();
        let api = crate::filters::game_api(teams_db.clone());

        let res = warp::test::request()
            .path("/teams")
            .reply(&api).await;

        assert_eq!(res.status(), http::StatusCode::OK);
        assert_eq!(res.body(), "[]");
    }

    #[tokio::test]
    async fn test_add_team() {
        let teams_db = TeamsDb::new();
        let api = crate::filters::game_api(teams_db.clone());

        let new_team = Team{ name: "first_team".into(), participants: vec!["ori".to_owned()] };

        let res = warp::test::request()
            .path("/register_team")
            .method("POST")
            .json(&new_team)
            .reply(&api).await;

        assert_eq!(res.status(), http::StatusCode::OK);

        assert!(teams_db.contains(&new_team.name).await);
    }


    #[tokio::test]
    async fn test_team_access() {
        println!("Start test ababab");
        use hex_string::HexString;
        let teams_db = TeamsDb::new();
        let api = crate::filters::game_api(teams_db.clone());

        let new_team = Team { name: "first_team".into(), participants: vec!["ori".to_owned()] };

        let res = warp::test::request()
            .path("/register_team")
            .method("POST")
            .json(&new_team)
            .reply(&api).await;

        assert_eq!(res.status(), http::StatusCode::OK, "team registration failed");
        let team_token: TeamToken = serde_json::from_slice(res.body())
            .expect("should receive token");

        let submit_path = format!("/team/{}/{}/submit", new_team.name, HexString::from_bytes(&team_token.token).as_str());
        let res = warp::test::request()
            .method("POST")
            .path(&submit_path)
            .reply(&api).await;

        assert_eq!(res.status(), http::StatusCode::OK, "failed to submit as team in {} with body {:?}", submit_path, res.body());
        assert_eq!(res.body(), "");
    }

}
