use super::{Team, TeamsDb};
use crate::team::TeamName;
use crate::{sign_on_team_name, AccessGranted, TeamToken};
use hex_string::HexString;

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

pub fn submit_solution(team_accessed: AccessGranted) -> impl warp::Reply {
    format!("Legit submission to {}", team_accessed.team)
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
