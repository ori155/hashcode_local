use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use crate::models::TeamName;
use crate::models::solution::{InputFileName, ChallengeDate};

pub type Score = u64;

#[derive(Clone)]
pub struct ScoreBoard {
    db: Arc<RwLock<HashMap<ChallengeDate, HashMap<TeamName, HashMap<InputFileName, Score>>>>>
}

impl ScoreBoard {
    pub fn new() -> Self {
        Self { db: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn add_team_score(&mut self, team_name: &TeamName, file_name: &InputFileName, score: Score, challenge: ChallengeDate) {
        log::info!("Challenge {}: Team '{}' scored {} on file {}", challenge, team_name, score, file_name);
        let mut db_write_access = self.db.write().await;
        let stored_score = db_write_access
            .entry(challenge.clone())
            .or_default()
            .entry(team_name.clone())
            .or_default()
            .entry(file_name.clone())
            .or_default();

        if score > *stored_score {
            *stored_score = score;
        }

    }

    pub async fn best_per_input(&self, team_name: &TeamName, challenge: ChallengeDate) -> HashMap<InputFileName, Score> {
        if let Some(team_to_input_to_score) = self.db.write().await.get(&challenge) {
            if let Some(input_to_score) = team_to_input_to_score.get(&team_name) {
                input_to_score.clone()
            } else {HashMap::new()}
        } else { HashMap::new() }
    }

    pub async fn total_score(&self, team_name: &TeamName, challenge: ChallengeDate) -> Score {
        self.best_per_input(team_name, challenge).await
            .values().sum()
    }
}

#[cfg(test)]
mod tests {
    use crate::scoreboard::ScoreBoard;
    use crate::models::TeamName;
    use crate::models::solution::ChallengeDate;

    #[tokio::test]
    async fn can_add_team() {

        let team = TeamName::from("abc");
        let input_file_name = "a".into();
        let challenge = ChallengeDate::Qualification(2020);

        let mut score_board = ScoreBoard::new();
        score_board.add_team_score(&team, &input_file_name, 120, challenge.clone()).await;

    }

    #[tokio::test]
    async fn can_get_best_total_score() {

        let team = TeamName::from("abc");
        let input_file_name = "a".into();
        let challenge = ChallengeDate::Qualification(2020);

        let mut score_board = ScoreBoard::new();
        score_board.add_team_score(&team, &input_file_name, 120, challenge.clone()).await;

        assert_eq!(
            score_board.total_score(&team, challenge.clone()).await,
            120
        )
    }
}
