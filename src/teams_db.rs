use std::collections::HashMap;
use std::sync::Arc;
use tokio::sync::RwLock;

use crate::models::{Team, TeamName};

#[derive(Clone)]
pub struct TeamsDb {
    inner: Arc<RwLock<HashMap<TeamName, Team>>>,
}

impl TeamsDb {
    pub fn new() -> Self {
        Self {
            inner: Arc::new(RwLock::new(HashMap::new())),
        }
    }

    pub async fn contains(&self, team_name: &TeamName) -> bool {
        self.inner.read().await.contains_key(team_name)
    }

    pub async fn insert(&mut self, team: Team) {
        let key = team.name.clone();
        self.inner.write().await.insert(key, team);
    }

    //TODO change to iterator
    pub async fn list_team_names(&self) -> Vec<TeamName> {
        self.inner.read().await.keys().map(|s| s.clone()).collect()
    }
}
