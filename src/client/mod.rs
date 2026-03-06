pub mod pagination;
pub mod request;
pub mod response;
pub mod retry;

use reqwest::Client;

use crate::auth::AuthProvider;
use crate::error::AppError;
use crate::models::ListResponse;
use crate::models::filter::FilterRequest;

pub struct CloudAppsClient {
    http: Client,
    base_url: String,
    auth: Box<dyn AuthProvider>,
}

impl CloudAppsClient {
    pub fn new(base_url: String, auth: Box<dyn AuthProvider>) -> Result<Self, AppError> {
        let http = Client::builder()
            .build()
            .map_err(|e| AppError::Network(format!("failed to build HTTP client: {}", e)))?;
        Ok(Self {
            http,
            base_url: base_url.trim_end_matches('/').to_string(),
            auth,
        })
    }

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    /// Send a GET request to the given path.
    pub async fn get(&self, path: &str) -> Result<reqwest::Response, AppError> {
        let url = format!("{}{}", self.base_url, path);
        let token = self.auth.token()?;
        let response = retry::with_retry(|| async {
            self.http
                .get(&url)
                .header("Authorization", format!("Token {}", token))
                .send()
                .await
        })
        .await?;
        response::check_response(response).await
    }

    /// Send a POST request with a JSON body.
    pub async fn post(
        &self,
        path: &str,
        body: &FilterRequest,
    ) -> Result<reqwest::Response, AppError> {
        let url = format!("{}{}", self.base_url, path);
        let token = self.auth.token()?;
        let response = retry::with_retry(|| async {
            self.http
                .post(&url)
                .header("Authorization", format!("Token {}", token))
                .json(body)
                .send()
                .await
        })
        .await?;
        response::check_response(response).await
    }

    /// Send a POST request with a raw JSON value body.
    pub async fn post_json(
        &self,
        path: &str,
        body: &serde_json::Value,
    ) -> Result<reqwest::Response, AppError> {
        let url = format!("{}{}", self.base_url, path);
        let token = self.auth.token()?;
        let response = retry::with_retry(|| async {
            self.http
                .post(&url)
                .header("Authorization", format!("Token {}", token))
                .json(body)
                .send()
                .await
        })
        .await?;
        response::check_response(response).await
    }

    /// Send a DELETE request.
    pub async fn delete(&self, path: &str) -> Result<reqwest::Response, AppError> {
        let url = format!("{}{}", self.base_url, path);
        let token = self.auth.token()?;
        let response = retry::with_retry(|| async {
            self.http
                .delete(&url)
                .header("Authorization", format!("Token {}", token))
                .send()
                .await
        })
        .await?;
        response::check_response(response).await
    }

    /// List resources with automatic pagination.
    pub async fn list_all<T: serde::de::DeserializeOwned>(
        &self,
        path: &str,
        mut filter: FilterRequest,
    ) -> Result<Vec<T>, AppError> {
        let mut all_data: Vec<T> = Vec::new();
        let limit = filter.limit.unwrap_or(100);
        let mut skip = filter.skip.unwrap_or(0);

        loop {
            filter.skip = Some(skip);
            filter.limit = Some(limit);

            let resp = self.post(path, &filter).await?;
            let list: ListResponse<T> = resp.json().await?;

            let count = list.data.len() as u64;
            all_data.extend(list.data);

            if !list.has_next || count == 0 {
                break;
            }
            skip += count;
        }

        Ok(all_data)
    }
}
