pub mod team;
pub mod team_name;

pub mod solution {
    use serde_derive::{Deserialize, Serialize};

    #[derive(Serialize, Deserialize, Debug)]
    pub enum Challenge {
        Qual2020
    }

    pub type InputId = String;


    #[derive(Serialize, Deserialize, Debug)]
    pub struct Solution {
        pub challenge: Challenge,
        pub solutions: std::collections::HashMap<InputId, String>
    }
}

pub use team::Team;
pub use team_name::TeamName;
