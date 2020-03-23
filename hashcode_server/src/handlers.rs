use std::collections::HashMap;
use hex_string::HexString;
use crate::teams_db::TeamsDb;
use crate::models::{TeamName, Team};
use crate::{sign_on_team_name, verify_team_token};
use crate::scoreboard::ScoreBoard;
use crate::models::solution::{InputFileName, ChallengeDate, SolutionSubmitRequest};

pub async fn add_team(
    new_team: Team,
    mut teams_db: TeamsDb,
) -> Result<impl warp::Reply, std::convert::Infallible> {
    if teams_db.contains(&new_team.name).await {
        return Ok(warp::reply::json(&crate::ApiError::ErrorTeamExists));
    }

    let new_team_token = sign_on_team_name(&new_team.name);

    log::info!("Team '{}' was registered with token {}", new_team.name,
                HexString::from_bytes(&new_team_token.token).as_str());

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
pub struct WrongToken;
impl warp::reject::Reject for WrongToken {}

#[derive(Debug)]
pub struct UnknownChallenge;
impl warp::reject::Reject for UnknownChallenge {}

#[derive(Debug)]
pub struct BadSubmission(hashcode_score_calc::ScoringError);
impl warp::reject::Reject for BadSubmission {}

use hashcode_score_calc::Challenge;
use std::sync::Arc;

pub async fn submit_solution(solution_req: SolutionSubmitRequest, challenges: Arc<Vec<Challenge>>, mut scoreboard: ScoreBoard) -> Result<impl warp::Reply, warp::Rejection> {
    use hashcode_score_calc::Score;

    if !verify_team_token(&solution_req.token.into(), &solution_req.team_name) {
        return Err(warp::reject::custom(WrongToken));
    }

    let SolutionSubmitRequest{solution, team_name, ..} = solution_req;

    let new_scores = {

        let relevant_challenge = challenges.iter()
            .find(|&c| c.date == solution.challenge)
            .ok_or(warp::reject::custom(UnknownChallenge))?;

        let mut new_scores = HashMap::<InputFileName, Score>::new();
        for input_file_name in &relevant_challenge.input_file_names {
            let submission = match solution.solutions.get(input_file_name) {
                None => { continue },
                Some(sub) => sub,
            };

            let score = (relevant_challenge.score_function)(submission, input_file_name)
                .map_err(|e| warp::reject::custom(BadSubmission(e)))?;

            new_scores.insert(input_file_name.clone(), score);
        }
        new_scores
    };

    for (input_file_name, score) in &new_scores {
        scoreboard.add_team_score(&team_name, input_file_name, *score, solution.challenge.clone()).await;
    }

    Ok(warp::reply::json(&new_scores))
}

pub async fn view_scoreboard(challenge_date: ChallengeDate, scoreboard: ScoreBoard, teams: TeamsDb) -> Result<impl warp::Reply, std::convert::Infallible> {

    let mut score_view = HashMap::new();
    for tn in teams.list_team_names().await {
        let best_score_of_team = scoreboard.total_score(&tn, challenge_date.clone()).await;
        score_view.insert(tn, best_score_of_team);
    }

    Ok(warp::reply::json(&score_view))
}

pub async fn handle_submit_rejection(rej: warp::Rejection) -> Result<impl warp::Reply, warp::Rejection> {

    if let Some(UnknownChallenge) = rej.find() {
        Ok("It seems like you're trying to play an unimplemented game".to_owned())
    } else  if let Some(BadSubmission(scoring_err)) = rej.find() {
        Ok(format!("{}", scoring_err))
    } else {
        Err(rej)
    }
}

