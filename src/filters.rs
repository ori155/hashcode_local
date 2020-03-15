use crate::models::TeamName;
use crate::teams_db::TeamsDb;
use crate::AccessGranted;
use hex_string::HexString;
use warp::Filter;
use crate::scoreboard::ScoreBoard;

fn with_db(
    db: TeamsDb,
) -> impl Filter<Extract = (TeamsDb,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn with_scoreboard(
    db: ScoreBoard,
) -> impl Filter<Extract = (ScoreBoard,), Error = std::convert::Infallible> + Clone {
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

pub fn submit_solution(scoreboard: ScoreBoard) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    warp::post()
        .and(team_access())
        .and(warp::path::path("submit"))
        .and(with_scoreboard(scoreboard))
        .and(warp::body::json())
        .and_then(crate::handlers::submit_solution)
}

pub fn view_scoreboard(scoreboard: ScoreBoard) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    warp::get()
        .and(warp::path::path("scoreboard"))
        .and(with_scoreboard(scoreboard))
        .and_then(crate::handlers::view_scoreboard)
}

pub fn game_api(
    teams: TeamsDb,
    scoreboard: ScoreBoard
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    team_registration(teams.clone())
        .or(list_teams(teams.clone()))
        .or(submit_solution(scoreboard.clone()))
        .or(view_scoreboard(scoreboard))
}
