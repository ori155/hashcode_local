use warp::Filter;
use std::collections::{HashMap, HashSet};
use serde_derive::{Deserialize, Serialize};
use tokio::sync::{RwLock, Mutex};
use std::sync::Arc;

#[derive(Serialize, Deserialize, Debug)]
pub struct Team {
    name: String,
    participants: Vec<String>
}

#[derive(Debug)]
struct ExistingTeam;
impl warp::reject::Reject for ExistingTeam {}

type TeamsDb = Arc<RwLock<HashSet<String>>>;

fn with_db(db: TeamsDb) -> impl Filter<Extract = (TeamsDb,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

mod handlers {
    use super::{Team, TeamsDb};
    pub async fn add_team(new_team: Team, teams_db: TeamsDb) -> Result<impl warp::Reply, std::convert::Infallible> {
        let mut w_teams = teams_db.write().await;

        if w_teams.contains(&new_team.name) {
            return Ok("Err: Team Exists")
        }

        w_teams.insert(new_team.name);

        Ok("New team added")
    }

    pub async fn list_teams(teams_db: TeamsDb) -> Result<impl warp::Reply, std::convert::Infallible> {
        let listed_teams: Vec<String> = {
            let r_teams = teams_db.read().await;
            r_teams.iter().map(|s| s.clone()).collect()
        };

        Ok(format!("Teams: {:?}", listed_teams))
    }
}

#[tokio::main]
async fn main() {

    let teams = Arc::new(RwLock::new(HashSet::<String, _>::new()));



    let team_registration = warp::post()
        .and(warp::body::json())
        .and(with_db(teams.clone()))
        .and_then( handlers::add_team);

    let list_teams = warp::get()
        .and(warp::path("teams"))
        .and(with_db(teams.clone()))
        .and_then(handlers::list_teams);

    let routes = team_registration.or(list_teams);

    warp::serve(routes).run(([127, 0, 0, 1], 8080)).await;

}
