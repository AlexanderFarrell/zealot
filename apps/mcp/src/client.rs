use reqwest::{Client, StatusCode};
use serde::{Serialize, de::DeserializeOwned};
use thiserror::Error;

#[derive(Debug, Error)]
pub enum ApiError {
    #[error("not found")]
    NotFound,
    #[error("HTTP {status}: {message}")]
    Http { status: StatusCode, message: String },
    #[error(transparent)]
    Request(#[from] reqwest::Error),
}

#[derive(Clone)]
pub struct ZealotClient {
    inner: Client,
    base_url: String,
    api_key: String,
}

impl ZealotClient {
    pub fn new(base_url: impl Into<String>, api_key: impl Into<String>) -> Self {
        Self {
            inner: Client::new(),
            base_url: base_url.into().trim_end_matches('/').to_string(),
            api_key: api_key.into(),
        }
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let resp = self
            .inner
            .get(self.url(path))
            .header("X-API-Key", &self.api_key)
            .send()
            .await?;
        self.parse(resp).await
    }

    pub async fn post<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let resp = self
            .inner
            .post(self.url(path))
            .header("X-API-Key", &self.api_key)
            .json(body)
            .send()
            .await?;
        self.parse(resp).await
    }

    pub async fn patch<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let resp = self
            .inner
            .patch(self.url(path))
            .header("X-API-Key", &self.api_key)
            .json(body)
            .send()
            .await?;
        self.parse(resp).await
    }

    pub async fn put<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let resp = self
            .inner
            .put(self.url(path))
            .header("X-API-Key", &self.api_key)
            .json(body)
            .send()
            .await?;
        self.parse(resp).await
    }

    pub async fn post_no_response<B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<(), ApiError> {
        let resp = self
            .inner
            .post(self.url(path))
            .header("X-API-Key", &self.api_key)
            .json(body)
            .send()
            .await?;
        if resp.status() == StatusCode::NOT_FOUND {
            return Err(ApiError::NotFound);
        }
        if !resp.status().is_success() {
            let status = resp.status();
            let message = resp.text().await.unwrap_or_default();
            return Err(ApiError::Http { status, message });
        }
        Ok(())
    }

    pub async fn put_no_response<B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<(), ApiError> {
        let resp = self
            .inner
            .put(self.url(path))
            .header("X-API-Key", &self.api_key)
            .json(body)
            .send()
            .await?;
        if resp.status() == StatusCode::NOT_FOUND {
            return Err(ApiError::NotFound);
        }
        if !resp.status().is_success() {
            let status = resp.status();
            let message = resp.text().await.unwrap_or_default();
            return Err(ApiError::Http { status, message });
        }
        Ok(())
    }

    pub async fn patch_no_response<B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<(), ApiError> {
        let resp = self
            .inner
            .patch(self.url(path))
            .header("X-API-Key", &self.api_key)
            .json(body)
            .send()
            .await?;
        if resp.status() == StatusCode::NOT_FOUND {
            return Err(ApiError::NotFound);
        }
        if !resp.status().is_success() {
            let status = resp.status();
            let message = resp.text().await.unwrap_or_default();
            return Err(ApiError::Http { status, message });
        }
        Ok(())
    }

    pub async fn delete(&self, path: &str) -> Result<(), ApiError> {
        let resp = self
            .inner
            .delete(self.url(path))
            .header("X-API-Key", &self.api_key)
            .send()
            .await?;
        if resp.status() == StatusCode::NOT_FOUND {
            return Err(ApiError::NotFound);
        }
        if !resp.status().is_success() {
            let status = resp.status();
            let message = resp.text().await.unwrap_or_default();
            return Err(ApiError::Http { status, message });
        }
        Ok(())
    }

    async fn parse<T: DeserializeOwned>(&self, resp: reqwest::Response) -> Result<T, ApiError> {
        let status = resp.status();
        if status == StatusCode::NOT_FOUND {
            return Err(ApiError::NotFound);
        }
        let body = resp.text().await.unwrap_or_default();
        if !status.is_success() {
            return Err(ApiError::Http {
                status,
                message: body,
            });
        }
        serde_json::from_str(&body).map_err(|e| ApiError::Http {
            status,
            message: format!("JSON decode failed: {e} — raw body: {body}"),
        })
    }
}
