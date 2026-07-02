use reqwest::{Client, Method, StatusCode, header};
use serde::{Serialize, de::DeserializeOwned};

use crate::error::ApiError;

/// A downloaded binary payload (media file, PDF/DOCX export).
#[derive(Debug, Clone)]
pub struct Download {
    pub bytes: Vec<u8>,
    pub content_type: Option<String>,
    /// Filename suggested by the server via Content-Disposition, if any.
    pub filename: Option<String>,
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

    pub fn base_url(&self) -> &str {
        &self.base_url
    }

    fn url(&self, path: &str) -> String {
        format!("{}{}", self.base_url, path)
    }

    fn request(&self, method: Method, path: &str) -> reqwest::RequestBuilder {
        self.inner
            .request(method, self.url(path))
            .header("X-API-Key", &self.api_key)
    }

    pub async fn get<T: DeserializeOwned>(&self, path: &str) -> Result<T, ApiError> {
        let resp = self.request(Method::GET, path).send().await?;
        self.parse(resp).await
    }

    pub async fn post<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let resp = self.request(Method::POST, path).json(body).send().await?;
        self.parse(resp).await
    }

    pub async fn patch<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let resp = self.request(Method::PATCH, path).json(body).send().await?;
        self.parse(resp).await
    }

    pub async fn put<B: Serialize, T: DeserializeOwned>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<T, ApiError> {
        let resp = self.request(Method::PUT, path).json(body).send().await?;
        self.parse(resp).await
    }

    pub async fn post_no_response<B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<(), ApiError> {
        let resp = self.request(Method::POST, path).json(body).send().await?;
        Self::check_status(resp).await
    }

    /// POST with no request body and no meaningful response (e.g. rebuild-links
    /// returns JSON we may ignore, assign_type returns a bare status).
    pub async fn post_empty(&self, path: &str) -> Result<(), ApiError> {
        let resp = self.request(Method::POST, path).send().await?;
        Self::check_status(resp).await
    }

    pub async fn put_no_response<B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<(), ApiError> {
        let resp = self.request(Method::PUT, path).json(body).send().await?;
        Self::check_status(resp).await
    }

    pub async fn patch_no_response<B: Serialize>(
        &self,
        path: &str,
        body: &B,
    ) -> Result<(), ApiError> {
        let resp = self.request(Method::PATCH, path).json(body).send().await?;
        Self::check_status(resp).await
    }

    pub async fn delete(&self, path: &str) -> Result<(), ApiError> {
        let resp = self.request(Method::DELETE, path).send().await?;
        Self::check_status(resp).await
    }

    /// Download raw bytes (media files, exports). Captures Content-Type and the
    /// Content-Disposition filename when the server suggests one.
    pub async fn get_bytes(&self, path: &str) -> Result<Download, ApiError> {
        let resp = self.request(Method::GET, path).send().await?;
        let status = resp.status();
        if status == StatusCode::NOT_FOUND {
            return Err(ApiError::NotFound);
        }
        if !status.is_success() {
            let message = resp.text().await.unwrap_or_default();
            return Err(ApiError::Http { status, message });
        }
        let content_type = resp
            .headers()
            .get(header::CONTENT_TYPE)
            .and_then(|v| v.to_str().ok())
            .map(String::from);
        let filename = resp
            .headers()
            .get(header::CONTENT_DISPOSITION)
            .and_then(|v| v.to_str().ok())
            .and_then(parse_disposition_filename);
        let bytes = resp.bytes().await?.to_vec();
        Ok(Download {
            bytes,
            content_type,
            filename,
        })
    }

    /// Upload a file as a multipart form (used by POST /media/{path}).
    pub async fn post_multipart(
        &self,
        path: &str,
        field_name: &str,
        filename: &str,
        bytes: Vec<u8>,
    ) -> Result<(), ApiError> {
        let part = reqwest::multipart::Part::bytes(bytes).file_name(filename.to_string());
        let form = reqwest::multipart::Form::new().part(field_name.to_string(), part);
        let resp = self
            .request(Method::POST, path)
            .multipart(form)
            .send()
            .await?;
        Self::check_status(resp).await
    }

    /// Raw escape hatch: send an arbitrary request and return the response body
    /// text with its status. Used by `zealot api`.
    pub async fn raw(
        &self,
        method: Method,
        path: &str,
        body: Option<serde_json::Value>,
    ) -> Result<(StatusCode, String), ApiError> {
        let mut req = self.request(method, path);
        if let Some(body) = body {
            req = req.json(&body);
        }
        let resp = req.send().await?;
        let status = resp.status();
        let text = resp.text().await.unwrap_or_default();
        Ok((status, text))
    }

    async fn check_status(resp: reqwest::Response) -> Result<(), ApiError> {
        let status = resp.status();
        if status == StatusCode::NOT_FOUND {
            return Err(ApiError::NotFound);
        }
        if !status.is_success() {
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

/// Extract `filename="…"` from a Content-Disposition header value.
fn parse_disposition_filename(value: &str) -> Option<String> {
    let idx = value.find("filename=")?;
    let raw = &value[idx + "filename=".len()..];
    let raw = raw.split(';').next().unwrap_or(raw).trim();
    Some(raw.trim_matches('"').to_string()).filter(|s| !s.is_empty())
}

#[cfg(test)]
mod tests {
    use super::parse_disposition_filename;

    #[test]
    fn parses_quoted_disposition_filename() {
        assert_eq!(
            parse_disposition_filename("attachment; filename=\"My Note.pdf\""),
            Some("My Note.pdf".to_string())
        );
    }

    #[test]
    fn parses_unquoted_disposition_filename() {
        assert_eq!(
            parse_disposition_filename("attachment; filename=note.docx"),
            Some("note.docx".to_string())
        );
    }

    #[test]
    fn missing_filename_yields_none() {
        assert_eq!(parse_disposition_filename("inline"), None);
    }
}
