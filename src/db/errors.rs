use actix_web::{error};
use derive_more::{Display, Error};

// Generic error for if shit goes real bad
#[derive(Debug, Clone, Display, Error)]
#[display(fmt = "Database error")]
pub struct DatabaseError;

impl error::ResponseError for DatabaseError {}

impl From<std::io::Error> for DatabaseError {
    fn from(_value: std::io::Error) -> Self {
        DatabaseError{}
    }
}

// Error when attempting to switch to a world that doesn't/hasn't existed 
#[derive(Debug, Clone, Display, Error)]
#[display(fmt = "WorldNotFoundError: world {world_number} does not exist")]
pub struct WorldNotFoundError {
    pub world_number: u64
}

impl error::ResponseError for WorldNotFoundError {}