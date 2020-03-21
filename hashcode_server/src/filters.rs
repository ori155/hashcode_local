use crate::models::TeamName;
use crate::teams_db::TeamsDb;
use crate::AccessGranted;
use hex_string::HexString;
use warp::Filter;
use crate::scoreboard::ScoreBoard;
use hashcode_score_calc::Challenge;
use std::sync::Arc;
use crate::models::solution::ChallengeDate;
use crate::handlers::UnknownChallenge;

fn with_db(
    db: TeamsDb,
) -> impl Filter<Extract = (TeamsDb,), Error = std::convert::Infallible> + Clone {
    warp::any().map(move || db.clone())
}

fn with_challenges(
    challenges: Vec<Challenge>,
) -> impl Filter<Extract = (Arc<Vec<Challenge>>,), Error = std::convert::Infallible> + Clone {
    let ac = Arc::new(challenges);
    warp::any().map(move || ac.clone())
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

pub fn submit_solution(scoreboard: ScoreBoard) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    warp::post()
        .and(warp::body::json())
        .and(warp::path::path("submit"))
        .and(with_challenges(hashcode_score_calc::get_challenges()))
        .and(with_scoreboard(scoreboard))
        .and_then(crate::handlers::submit_solution)
        .recover(crate::handlers::handle_submit_rejection)
}


//TODO: This is a quick and dirty hack
pub fn challenge_data_from_path() -> impl Filter<Extract = (ChallengeDate,), Error = warp::Rejection> + Clone
{
    async fn handle(s: String) -> Result<ChallengeDate, warp::Rejection> {
        match s.as_str() {
            "qual2020" => Ok(ChallengeDate::Qualification(2020)),
            "qual2016" => Ok(ChallengeDate::Qualification(2016)),
            _ => Err(warp::reject::custom(UnknownChallenge))
        }
    }
        warp::path::param::<String>()
       .and_then(handle)
}

pub fn view_scoreboard(scoreboard: ScoreBoard, teams: TeamsDb) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone
{
    warp::get()
        .and(warp::path::path("scoreboard"))
        .and(challenge_data_from_path())
        .and(with_scoreboard(scoreboard))
        .and(with_db(teams))
        .and_then(crate::handlers::view_scoreboard)
}

pub fn game_api(
    teams: TeamsDb,
    scoreboard: ScoreBoard
) -> impl Filter<Extract = impl warp::Reply, Error = warp::Rejection> + Clone {
    team_registration(teams.clone())
        .with(warp::log("team-registration"))
        .or(list_teams(teams.clone()))
        .or(submit_solution(scoreboard.clone()))
        .or(view_scoreboard(scoreboard, teams.clone())
            .with(warp::log("scoreboard"))
        )
        .or(warp::fs::dir("static")
            .with(warp::log("static-serv")))
        .or(warp::get()
            .and(warp::path::end())
            .and(warp::fs::file("static/submission.html"))
            .with(warp::log("submission-html"))
        )
}
