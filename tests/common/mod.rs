use cloudapps::auth::AuthProvider;
use cloudapps::client::CloudAppsClient;
use cloudapps::error::AppError;

pub struct MockAuth;

impl AuthProvider for MockAuth {
    fn token(&self) -> Result<String, AppError> {
        Ok("test-token".to_string())
    }
}

pub fn create_client(base_url: &str) -> CloudAppsClient {
    CloudAppsClient::new(base_url.to_string(), Box::new(MockAuth)).unwrap()
}
