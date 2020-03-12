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

    use crate::team::Team;

    #[derive(Clone)]
    pub struct TeamsDb {
        inner: Arc<RwLock<HashMap<String, Team>>>
    }

    impl TeamsDb {
        pub fn new() -> Self {
            Self{inner: Arc::new(RwLock::new(HashMap::<String, Team>::new()))}
        }

        pub async fn contains(&self, team: &Team) -> bool {
            self.inner.read().await
                .contains_key(&team.name)
        }

        pub async fn insert(&mut self, team: Team) {
            let key = team.name.clone();
            self.inner.write().await
                .insert(key, team);
        }

        //TODO change to iterator
        pub async fn list_team_names(&self) -> Vec<String> {
            self.inner.read().await
                .keys().map(|s| s.to_owned()).collect()
        }
    }

}

use teams_db::TeamsDb;

fn with_db(db: TeamsDb) -> impl Filter<Extract = (TeamsDb,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

mod handlers {
    use super::{Team, TeamsDb};
    pub async fn add_team(new_team: Team, mut teams_db: TeamsDb) -> Result<impl warp::Reply, std::convert::Infallible> {

        if teams_db.contains(&new_team).await {
            return Ok("Err: Team Exists")
        }

        teams_db.insert(new_team).await;

        Ok("New team added")
    }

    pub async fn list_teams(teams_db: TeamsDb) -> Result<impl warp::Reply, std::convert::Infallible> {
        let listed_teams: Vec<String> = teams_db.list_team_names().await;

        Ok(format!("Teams: {:?}", listed_teams))
    }
}

#[tokio::main]
async fn main() {

    let teams = TeamsDb::new();

    let team_registration = warp::post()
        .and(warp::body::json())
        .and(with_db(teams.clone()))
        .and_then( handlers::add_team);

    let list_teams = warp::get()
        .and(warp::path("teams"))
        .and(warp::path::end())
        .and(with_db(teams.clone()))
        .and_then(handlers::list_teams);

    let routes = team_registration.or(list_teams);

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;

}
