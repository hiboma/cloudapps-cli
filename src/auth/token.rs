use crate::auth::AuthProvider;
use crate::error::AppError;

pub struct TokenAuth {
    token: String,
}

impl TokenAuth {
    pub fn new(token: String) -> Result<Self, AppError> {
        if token.is_empty() {
            return Err(AppError::Auth(
                "API token is empty. Set via --token, CLOUDAPPS_API_TOKEN, or config file."
                    .to_string(),
            ));
        }
        Ok(Self { token })
    }
}

impl AuthProvider for TokenAuth {
    fn token(&self) -> Result<String, AppError> {
        Ok(self.token.clone())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_token_auth_valid() {
        let auth = TokenAuth::new("test-token".to_string()).unwrap();
        assert_eq!(auth.token().unwrap(), "test-token");
    }

    #[test]
    fn test_token_auth_empty() {
        let result = TokenAuth::new(String::new());
        assert!(result.is_err());
    }
}
