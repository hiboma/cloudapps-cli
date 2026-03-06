use thiserror::Error;

#[derive(Debug, Error)]
pub enum AppError {
    #[error("authentication error: {0}")]
    Auth(String),

    #[error("API error (status {status}): {message}")]
    Api { status: u16, message: String },

    #[error("network error: {0}")]
    Network(String),

    #[error("invalid input: {0}")]
    InvalidInput(String),

    #[error("configuration error: {0}")]
    Config(String),

    #[error("rate limited, retry after backoff")]
    RateLimited,

    #[error(transparent)]
    Http(#[from] reqwest::Error),

    #[error(transparent)]
    Json(#[from] serde_json::Error),

    #[error(transparent)]
    Io(#[from] std::io::Error),
}

impl AppError {
    pub fn exit_code(&self) -> i32 {
        match self {
            AppError::Auth(_) => 2,
            AppError::Api { .. } => 3,
            AppError::Network(_) | AppError::Http(_) => 4,
            AppError::InvalidInput(_) => 5,
            _ => 1,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exit_code_auth() {
        let err = AppError::Auth("bad token".to_string());
        assert_eq!(err.exit_code(), 2);
    }

    #[test]
    fn test_exit_code_api() {
        let err = AppError::Api {
            status: 404,
            message: "not found".to_string(),
        };
        assert_eq!(err.exit_code(), 3);
    }

    #[test]
    fn test_exit_code_network() {
        let err = AppError::Network("timeout".to_string());
        assert_eq!(err.exit_code(), 4);
    }

    #[test]
    fn test_exit_code_invalid_input() {
        let err = AppError::InvalidInput("bad value".to_string());
        assert_eq!(err.exit_code(), 5);
    }

    #[test]
    fn test_exit_code_rate_limited() {
        let err = AppError::RateLimited;
        assert_eq!(err.exit_code(), 1);
    }

    #[test]
    fn test_exit_code_config() {
        let err = AppError::Config("missing file".to_string());
        assert_eq!(err.exit_code(), 1);
    }

    #[test]
    fn test_display_auth() {
        let err = AppError::Auth("bad token".to_string());
        assert_eq!(err.to_string(), "authentication error: bad token");
    }

    #[test]
    fn test_display_api() {
        let err = AppError::Api {
            status: 500,
            message: "internal error".to_string(),
        };
        assert_eq!(err.to_string(), "API error (status 500): internal error");
    }

    #[test]
    fn test_display_rate_limited() {
        let err = AppError::RateLimited;
        assert_eq!(err.to_string(), "rate limited, retry after backoff");
    }
}
