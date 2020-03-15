use crate::teams_db::TeamsDb;
use crate::models::{TeamName, Team};
use crate::{sign_on_team_name, AccessGranted, TeamToken};
use hex_string::HexString;
use crate::scoreboard::{ScoreBoard, Score};
use std::collections::HashMap;
use crate::models::solution::Solution;
use std::error::Error;

pub async fn add_team(
    new_team: Team,
    mut teams_db: TeamsDb,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    if teams_db.contains(&new_team.name).await {
        return Ok(warp::reply::json(&crate::ApiError::ErrorTeamExists));
    }

    let new_team_token = sign_on_team_name(&new_team.name);
    teams_db.insert(new_team).await;

    Ok(warp::reply::json(&new_team_token))
}

pub async fn list_teams(teams_db: TeamsDb) -> Result<impl warp::Reply, std::convert::Infallible> {
    let listed_teams: Vec<TeamName> = teams_db.list_team_names().await;

    Ok(warp::reply::json(&listed_teams))
}

#[derive(Debug)]
pub struct UnknownInputCase;
impl warp::reject::Reject for UnknownInputCase {}

#[derive(Debug)]
pub struct UnknownChallenge;
impl warp::reject::Reject for UnknownChallenge {}

#[derive(Debug)]
pub struct BadSubmissionFormat(hashcode_score_calc::ScoringError);
impl warp::reject::Reject for BadSubmissionFormat {}

pub async fn submit_solution(team_accessed: AccessGranted, mut scoreboard: ScoreBoard, solution: Solution) -> Result<impl warp::Reply, warp::Rejection> {
    use crate::models::solution::ChallengeDate;
    use hashcode_score_calc::Score;

    let total_score = {
        //TODO: Cach the challenges
        let challenges = hashcode_score_calc::get_challenges();
        let relevant_challenge = challenges.iter()
            .find(|&c| c.date == solution.challenge)
            .ok_or(warp::reject::custom(UnknownChallenge))?;

        let mut total_score: Score = 0;
        for input_file_name in &relevant_challenge.input_file_names {
            let submission = match solution.solutions.get(input_file_name) {
                None => { continue },
                Some(sub) => sub,
            };

            total_score += (relevant_challenge.score_function)(submission, input_file_name)
                .map_err(|e| warp::reject::custom(BadSubmissionFormat(e)))?;
        }
        total_score
    };

    scoreboard.add_team_score(&team_accessed.team, total_score).await;
    Ok(warp::reply::json(&total_score))
}

pub async fn view_scoreboard(scoreboard: ScoreBoard, teams: TeamsDb) -> Result<impl warp::Reply, std::convert::Infallible> {

    let mut score_view = HashMap::new();
    for tn in teams.list_team_names().await {
        let best_score_of_team = scoreboard.get_best_score(&tn).await.unwrap_or(0);
        score_view.insert(tn, best_score_of_team);
    }

    Ok(warp::reply::json(&score_view))
}

pub async fn test_team_token(
    team_name: TeamName,
    team_token: HexString,
) -> Result<AccessGranted, warp::Rejection> {
    let team_token: TeamToken = team_token.into();

    if crate::verify_team_token(&team_token, &team_name) {
        Ok(AccessGranted { team: team_name })
    } else {
        Err(warp::reject::custom(crate::ApiError::WrongToken))
    }
}
