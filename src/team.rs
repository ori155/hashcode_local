use std::cmp;
use std::hash::{Hash, Hasher};
use serde_derive::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
pub struct Team {
    pub name: String,
    pub participants: Vec<String>
}


impl cmp::PartialEq for Team {
    fn eq(&self, other: &Team) -> bool {
        self.cmp(other) == cmp::Ordering::Equal
    }
}

impl cmp::Eq for Team {}

impl cmp::PartialOrd for Team {
    fn partial_cmp(&self, other: &Team) -> Option<cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl cmp::Ord for Team {
    fn cmp(&self, other: &Self) -> cmp::Ordering {
        self.name.trim().to_lowercase().cmp(&other.name.trim().to_lowercase())
    }
}

impl Hash for Team {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}
