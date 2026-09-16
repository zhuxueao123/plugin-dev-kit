use std::fs;
use std::path::Path;

use anyhow::{Context, Result};
use reqwest::blocking::multipart::{Form, Part};
use reqwest::blocking::{Client, Response};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use reqwest::StatusCode;
use serde::de::DeserializeOwned;
use serde_json::Value;

use crate::config::ResolvedConfig;

pub struct ApiClient {
    client: Client,
    config: ResolvedConfig,
}

impl ApiClient {
    pub fn new(config: ResolvedConfig) -> Result<Self> {
        let client = Client::builder()
            .build()
            .context("Failed to create HTTP client.")?;
        Ok(Self { client, config })
    }

    pub fn get_json<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response = self
            .client
            .get(self.url(path))
            .header(AUTHORIZATION, bearer_token(&self.config.token))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .with_context(|| format!("GET {} failed.", path))?;

        parse_json_response(response, path)
    }

    pub fn get_optional_json<T>(&self, path: &str) -> Result<Option<T>>
    where
        T: DeserializeOwned,
    {
        let response = self
            .client
            .get(self.url(path))
            .header(AUTHORIZATION, bearer_token(&self.config.token))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .with_context(|| format!("GET {} failed.", path))?;

        if response.status() == StatusCode::NOT_FOUND {
            return Ok(None);
        }

        parse_json_response(response, path).map(Some)
    }

    pub fn get_anonymous_json<T>(&self, path: &str) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response = self
            .client
            .get(self.url(path))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .with_context(|| format!("GET {} failed.", path))?;

        parse_json_response(response, path)
    }

    pub fn get_bytes(&self, path: &str) -> Result<Vec<u8>> {
        let response = self
            .client
            .get(self.url(path))
            .header(AUTHORIZATION, bearer_token(&self.config.token))
            .send()
            .with_context(|| format!("GET {} failed.", path))?;

        parse_bytes_response(response, path)
    }

    pub fn post_json<T>(&self, path: &str, payload: &Value) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response = self
            .client
            .post(self.url(path))
            .header(AUTHORIZATION, bearer_token(&self.config.token))
            .header(CONTENT_TYPE, "application/json")
            .json(payload)
            .send()
            .with_context(|| format!("POST {} failed.", path))?;

        parse_json_response(response, path)
    }

    pub fn post_json_bytes(&self, path: &str, payload: &Value) -> Result<Vec<u8>> {
        let response = self
            .client
            .post(self.url(path))
            .header(AUTHORIZATION, bearer_token(&self.config.token))
            .header(CONTENT_TYPE, "application/json")
            .json(payload)
            .send()
            .with_context(|| format!("POST {} failed.", path))?;

        parse_bytes_response(response, path)
    }

    pub fn post_multipart_json<T>(
        &self,
        path: &str,
        file_field: &str,
        file_path: &Path,
        fields: &[(&str, String)],
    ) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response = self
            .client
            .post(self.url(path))
            .header(AUTHORIZATION, bearer_token(&self.config.token))
            .multipart(build_multipart_form(file_field, file_path, fields)?)
            .send()
            .with_context(|| format!("POST {} failed.", path))?;

        parse_json_response(response, path)
    }

    pub fn post_multipart_bytes(
        &self,
        path: &str,
        file_field: &str,
        file_path: &Path,
        fields: &[(&str, String)],
    ) -> Result<Vec<u8>> {
        let response = self
            .client
            .post(self.url(path))
            .header(AUTHORIZATION, bearer_token(&self.config.token))
            .multipart(build_multipart_form(file_field, file_path, fields)?)
            .send()
            .with_context(|| format!("POST {} failed.", path))?;

        parse_bytes_response(response, path)
    }

    pub fn put_json<T>(&self, path: &str, payload: &Value) -> Result<T>
    where
        T: DeserializeOwned,
    {
        let response = self
            .client
            .put(self.url(path))
            .header(AUTHORIZATION, bearer_token(&self.config.token))
            .header(CONTENT_TYPE, "application/json")
            .json(payload)
            .send()
            .with_context(|| format!("PUT {} failed.", path))?;

        parse_json_response(response, path)
    }

    pub fn put_empty(&self, path: &str, payload: &Value) -> Result<DeleteResult> {
        let response = self
            .client
            .put(self.url(path))
            .header(AUTHORIZATION, bearer_token(&self.config.token))
            .header(CONTENT_TYPE, "application/json")
            .json(payload)
            .send()
            .with_context(|| format!("PUT {} failed.", path))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            anyhow::bail!(
                "Request to {} failed with status {}: {}",
                path,
                status,
                body
            );
        }

        Ok(DeleteResult {
            status: status.as_u16(),
            success: true,
        })
    }

    pub fn delete_empty(&self, path: &str) -> Result<DeleteResult> {
        let response = self
            .client
            .delete(self.url(path))
            .header(AUTHORIZATION, bearer_token(&self.config.token))
            .header(CONTENT_TYPE, "application/json")
            .send()
            .with_context(|| format!("DELETE {} failed.", path))?;

        let status = response.status();
        if !status.is_success() {
            let body = response.text().unwrap_or_default();
            anyhow::bail!(
                "Request to {} failed with status {}: {}",
                path,
                status,
                body
            );
        }

        Ok(DeleteResult {
            status: status.as_u16(),
            success: true,
        })
    }

    fn url(&self, path: &str) -> String {
        format!("{}/{}", self.config.base_url, path.trim_start_matches('/'))
    }
}

#[derive(Debug, serde::Serialize)]
pub struct DeleteResult {
    pub status: u16,
    pub success: bool,
}

fn parse_json_response<T>(response: Response, path: &str) -> Result<T>
where
    T: DeserializeOwned,
{
    let status = response.status();
    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        anyhow::bail!(
            "Request to {} failed with status {}: {}",
            path,
            status,
            body
        );
    }

    response
        .json::<T>()
        .with_context(|| format!("Failed to parse JSON response from {}.", path))
}

fn parse_bytes_response(response: Response, path: &str) -> Result<Vec<u8>> {
    let status = response.status();
    if !status.is_success() {
        let body = response.text().unwrap_or_default();
        anyhow::bail!(
            "Request to {} failed with status {}: {}",
            path,
            status,
            body
        );
    }

    response
        .bytes()
        .map(|bytes| bytes.to_vec())
        .with_context(|| format!("Failed to read binary response from {}.", path))
}

fn build_multipart_form(
    file_field: &str,
    file_path: &Path,
    fields: &[(&str, String)],
) -> Result<Form> {
    let file_name = file_path
        .file_name()
        .and_then(|value| value.to_str())
        .unwrap_or("upload.bin")
        .to_string();
    let bytes = fs::read(file_path)
        .with_context(|| format!("Failed to read file: {}", file_path.display()))?;
    let file_part = Part::bytes(bytes).file_name(file_name);
    let mut form = Form::new().part(file_field.to_string(), file_part);
    for (key, value) in fields {
        form = form.text((*key).to_string(), value.clone());
    }

    Ok(form)
}

fn bearer_token(token: &str) -> String {
    format!("Bearer {}", token)
}
