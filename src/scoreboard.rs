use std::sync::Arc;
use tokio::sync::RwLock;
use std::collections::HashMap;
use crate::models::TeamName;

pub type Score = u64;

#[derive(Clone)]
pub struct ScoreBoard {
    db: Arc<RwLock<HashMap<TeamName, Vec<Score>>>>
}

impl ScoreBoard {
    pub fn new() -> Self {
        Self { db: Arc::new(RwLock::new(HashMap::new())) }
    }

    pub async fn add_team_score(&mut self, team_name: &TeamName, score: Score) {
        self.db.write().await
            .entry(team_name.clone())
            .or_default()
            .push(score);
    }

    pub async fn get_best_score(&self, team_name: &TeamName) -> Option<Score> {
        self.db.read().await
            .get(team_name)
            .map(|score_vec| score_vec.iter().max().clone())
            .flatten()
            .map(|s| *s)
    }

    pub async fn best_scores(&self) -> HashMap<TeamName, Score> {
        self.db.read().await
            .iter()
            .map(|(name, scores): (&TeamName, &Vec<Score>)| (name.clone(), *scores.iter().max().unwrap_or(&0)))
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use crate::scoreboard::ScoreBoard;
    use crate::models::TeamName;

    #[tokio::test]
    async fn can_add_team() {

        let team = TeamName::from("abc");

        let mut score_board = ScoreBoard::new();
        score_board.add_team_score(&team, 120).await;

    }

    #[tokio::test]
    async fn can_get_best_score() {

        let team = TeamName::from("abc");

        let mut score_board = ScoreBoard::new();
        score_board.add_team_score(&team, 120).await;

        assert_eq!(
            score_board.get_best_score(&team).await,
            Some(120)
        )
    }
}
