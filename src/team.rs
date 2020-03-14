use serde_derive::{Deserialize, Serialize};
use std::cmp;
use std::hash::{Hash, Hasher};

pub use team_name::TeamName;

#[derive(Serialize, Deserialize, Debug)]
pub struct Team {
    pub name: TeamName,
    pub participants: Vec<String>,
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
        self.name.cmp(&other.name)
    }
}

impl Hash for Team {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.name.hash(state)
    }
}

mod team_name {
    use serde_derive::{Deserialize, Serialize};
    use std::{cmp, convert, fmt, hash};

    #[derive(Debug, Clone, Serialize, Deserialize)]
    pub struct TeamName(String);

    impl TeamName {
        pub fn as_str(&self) -> &str {
            self.0.as_str()
        }
    }

    impl fmt::Display for TeamName {
        fn fmt(&self, f: &mut fmt::Formatter) -> Result<(), fmt::Error> {
            self.0.fmt(f)
        }
    }

    impl TeamName {
        fn normalized(&self) -> String {
            self.0.trim().to_lowercase()
        }
    }

    impl cmp::PartialEq for TeamName {
        fn eq(&self, other: &TeamName) -> bool {
            self.normalized().cmp(&other.normalized()) == cmp::Ordering::Equal
        }
    }

    impl cmp::Ord for TeamName {
        fn cmp(&self, other: &Self) -> cmp::Ordering {
            self.normalized().cmp(&other.normalized())
        }
    }

    impl cmp::PartialOrd for TeamName {
        fn partial_cmp(&self, other: &Self) -> Option<cmp::Ordering> {
            Some(self.cmp(other))
        }
    }

    impl cmp::Eq for TeamName {}

    impl<T: convert::AsRef<str>> convert::From<T> for TeamName {
        fn from(t: T) -> Self {
            let r = t.as_ref();
            TeamName(r.into())
        }
    }

    impl hash::Hash for TeamName {
        fn hash<H: hash::Hasher>(&self, state: &mut H) {
            self.normalized().hash(state)
        }
    }

    impl std::str::FromStr for TeamName {
        type Err = std::convert::Infallible;

        fn from_str(s: &str) -> Result<Self, Self::Err> {
            Ok(s.into())
        }
    }
}
