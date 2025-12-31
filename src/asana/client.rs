//! HTTP client for Asana API requests.
//!
//! This module provides a low-level HTTP client wrapper for making requests
//! to the Asana API, handling authentication, pagination, and response parsing.

use super::models::*;
use anyhow::Result;
use reqwest::{Method, Response};

/// Makes requests to Asana and tries to conform response data to given model.
///
pub struct Client {
    pub(crate) access_token: String,
    pub(crate) base_url: String,
    endpoint: String,
    pub(crate) http_client: reqwest::Client,
}

impl Client {
    /// Returns a new instance for the given access token and base URL.
    ///
    /// # Panics
    /// Panics if the HTTP client cannot be created. This should never happen
    /// in practice as reqwest::Client::builder().build() only fails on
    /// invalid configuration, which we don't use.
    pub fn new(access_token: &str, base_url: &str) -> Self {
        Client {
            access_token: access_token.to_owned(),
            base_url: base_url.to_owned(),
            endpoint: String::from(""),
            http_client: reqwest::Client::builder()
                .build()
                .expect("Failed to create HTTP client - this should never happen"),
        }
    }

    /// Return model data for entity with GID or error.
    ///
    pub async fn get<T: Model>(&mut self, gid: &str) -> Result<T> {
        let model: Wrapper<T> = self
            .call::<T>(Method::GET, Some(gid), None)
            .await?
            .json()
            .await?;
        Ok(model.data)
    }

    /// Return vector of model data with pagination support.
    /// Uses Asana's token-based pagination as per https://developers.asana.com/docs/pagination
    ///
    pub async fn list_paginated<T: Model>(
        &mut self,
        params: Option<Vec<(&str, &str)>>,
        limit: Option<usize>,
    ) -> Result<Vec<T>> {
        let limit = limit.unwrap_or(100); // Default to 100 items per page (Asana's max)
        let mut all_data = Vec::new();
        let mut offset_token: Option<String> = None;
        let mut page = 0;

        loop {
            // Build params with owned strings stored in variables that live long enough
            let mut page_params_vec: Vec<(String, String)> = Vec::new();

            // Add original params if any (convert to owned strings)
            if let Some(ref original_params) = params {
                for (k, v) in original_params.iter() {
                    page_params_vec.push((k.to_string(), v.to_string()));
                }
            }

            // Add pagination parameters
            page_params_vec.push(("limit".to_string(), limit.to_string()));
            if let Some(ref offset) = offset_token {
                page_params_vec.push(("offset".to_string(), offset.clone()));
            }

            // Convert to string slices for the call
            let page_params: Vec<(&str, &str)> = page_params_vec
                .iter()
                .map(|(k, v)| (k.as_str(), v.as_str()))
                .collect();

            let response = self.call::<T>(Method::GET, None, Some(page_params)).await?;
            let status = response.status();

            // Check status before trying to deserialize
            if !status.is_success() {
                let response_text = response
                    .text()
                    .await
                    .unwrap_or_else(|_| String::from("Unable to read response"));
                log::error!(
                    "API request failed with status {}: {}",
                    status,
                    response_text
                );
                anyhow::bail!(
                    "API request failed with status {}: {}",
                    status,
                    response_text
                );
            }

            // Clone the response bytes so we can log them if deserialization fails
            let response_bytes = response.bytes().await?;
            let response_text = String::from_utf8_lossy(&response_bytes);

            // Try to deserialize, with better error message if it fails
            match serde_json::from_slice::<ListWrapper<T>>(&response_bytes) {
                Ok(model) => {
                    let page_data = model.data;
                    let page_size = page_data.len();
                    all_data.extend(page_data);

                    log::debug!(
                        "Fetched page {}: {} items (total so far: {})",
                        page,
                        page_size,
                        all_data.len()
                    );

                    // Asana pagination: if we got fewer items than the limit, we're done
                    if page_size < limit {
                        // Got fewer items than requested, we're done
                        break;
                    }

                    // Extract offset token from next_page for the next request
                    // Asana uses token-based pagination, not numeric offsets
                    offset_token = model.next_page.map(|next| next.offset);

                    // If there's no next_page token, we're done
                    if offset_token.is_none() {
                        break;
                    }

                    // If we got no items, we're done
                    if page_size == 0 {
                        break;
                    }

                    page += 1;
                }
                Err(e) => {
                    // Check if the response is empty or has a different structure
                    if response_bytes.is_empty() {
                        log::warn!("Received empty response from API, returning collected data");
                        self.endpoint.clear();
                        break;
                    }

                    // Try to parse as JSON to see if it's an error response
                    if let Ok(json_value) =
                        serde_json::from_slice::<serde_json::Value>(&response_bytes)
                    {
                        // Check if it's an error response
                        if json_value.get("errors").is_some() {
                            let errors = json_value.get("errors").and_then(|e| e.as_array());
                            log::error!("API returned errors: {:?}", errors);
                            anyhow::bail!("API returned errors: {:?}", errors);
                        }
                        // Check if it's missing the data field but otherwise valid JSON
                        if json_value.get("data").is_none() {
                            log::warn!(
                                "API response missing 'data' field, but otherwise valid. Response: {}",
                                response_text
                            );
                            self.endpoint.clear();
                            break;
                        }
                    }

                    log::error!(
                        "Failed to deserialize API response: {}. Response body: {}",
                        e,
                        response_text
                    );
                    anyhow::bail!(
                        "Failed to deserialize API response: {}. Response body: {}",
                        e,
                        response_text
                    );
                }
            }
        }

        self.endpoint.clear();
        log::debug!(
            "Completed paginated fetch: {} total items across {} pages",
            all_data.len(),
            page + 1
        );
        Ok(all_data)
    }

    /// Prepare endpoint for relational model data.
    ///
    pub(crate) fn from<T: Model>(&mut self, relational_gid: &str) -> &mut Client {
        self.endpoint = format!("{}/{}/", T::endpoint(), relational_gid);
        self
    }

    /// Make request and return response with model data or error.
    ///
    async fn call<T: Model>(
        &mut self,
        method: Method,
        gid: Option<&str>,
        params: Option<Vec<(&str, &str)>>,
    ) -> Result<Response> {
        self.call_with_body::<T>(method, gid, params, None).await
    }

    /// Make request with optional body and return response with model data or error.
    ///
    pub(crate) async fn call_with_body<T: Model>(
        &mut self,
        method: Method,
        gid: Option<&str>,
        params: Option<Vec<(&str, &str)>>,
        body: Option<serde_json::Value>,
    ) -> Result<Response> {
        // Add both relational and main endpoints, and entity gid if supplied
        let uri = format!("{}{}/", self.endpoint, T::endpoint());
        let uri = format!(
            "{}{}",
            uri,
            match gid {
                Some(gid) => gid.to_owned(),
                None => String::from(""),
            }
        );

        // Clear relational endpoint state
        self.endpoint.clear();

        // For PUT/DELETE/POST requests, don't add opt_fields as it can cause validation issues
        // opt_fields is primarily for GET requests to specify which fields to return
        // For PUT/POST requests, we only want to send the data in the body, not validate against opt_fields
        let mut uri = if matches!(method, Method::PUT | Method::DELETE | Method::POST) {
            // For PUT/DELETE/POST, don't add opt_fields - just use the base URI
            uri
        } else {
            // For GET only, add opt_fields to specify which fields to return
            let opts = format!(
                "this.({}),{}",
                T::field_names().join("|"),
                T::opt_strings().join(",")
            );
            format!("{}?opt_fields={}", uri, opts)
        };

        if let Some(params) = params {
            let separator = if uri.contains('?') { "&" } else { "?" };
            for param in params.iter() {
                uri = format!("{}{}{}={}", uri, separator, param.0, param.1);
            }
        }
        let request_url = format!("{}/{}", &self.base_url, uri);

        // Make request
        let mut request = self
            .http_client
            .request(method, &request_url)
            .header("Authorization", format!("Bearer {}", &self.access_token));

        if let Some(body) = body {
            request = request.json(&body);
        }

        Ok(request.send().await?)
    }
}
