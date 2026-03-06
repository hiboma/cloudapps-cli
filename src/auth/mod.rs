pub mod token;

use crate::error::AppError;

pub trait AuthProvider: Send + Sync {
    fn token(&self) -> Result<String, AppError>;
}
