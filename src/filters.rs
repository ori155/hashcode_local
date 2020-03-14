use crate::team::TeamName;
use crate::teams_db::TeamsDb;
use crate::AccessGranted;
use hex_string::HexString;
use warp::Filter;

fn with_db(
    db: TeamsDb,
) -> impl Filter<Extract = (TeamsDb,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

pub fn team_registration(
    teams: TeamsDb,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::post()
        .and(warp::path::path("register_team"))
        .and(warp::body::json())
        .and(with_db(teams))
        .and_then(crate::handlers::add_team)
}

pub fn list_teams(
    teams: TeamsDb,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    warp::get()
        .and(warp::path("teams"))
        .and(warp::path::end())
        .and(with_db(teams.clone()))
        .and_then(crate::handlers::list_teams)
}

pub fn team_access() -> impl Filter<Extract = (AccessGranted,), Error = warp::Rejection> + Clone {
    //Todo: test team exists
    warp::path!("team" / TeamName / HexString / ..).and_then(crate::handlers::test_team_token)
}

pub fn submit_solution() -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    warp::post()
        .and(team_access())
        .and(warp::path::path("submit"))
        .map(crate::handlers::submit_solution)
}

pub fn game_api(
    teams: TeamsDb,
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    team_registration(teams.clone())
        .or(list_teams(teams.clone()))
        .or(submit_solution())
}
