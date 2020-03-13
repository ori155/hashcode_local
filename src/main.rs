use warp::Filter;

pub mod team;
pub use team::Team;

#[derive(Debug)]
struct ExistingTeam;
impl warp::reject::Reject for ExistingTeam {}

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


mod handlers {
    use super::{Team, TeamsDb};
    use crate::team::TeamName;

    pub async fn add_team(new_team: Team, mut teams_db: TeamsDb) -> Result<impl warp::Reply, std::convert::Infallible> {

        if teams_db.contains(&new_team.name).await {
            return Ok("Err: Team Exists")
        }

        teams_db.insert(new_team).await;

        Ok("New team added")
    }

    pub async fn list_teams(teams_db: TeamsDb) -> Result<impl warp::Reply, std::convert::Infallible> {
        let listed_teams: Vec<TeamName> = teams_db.list_team_names().await;

        Ok(warp::reply::json(&listed_teams))
    }
}

mod filters {
    use crate::teams_db::TeamsDb;
    use warp::Filter;

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

    pub fn game_api(teams: TeamsDb) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
    {
        team_registration(teams.clone())
            .or(list_teams(teams.clone()))
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
    use crate::Team;

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

}
