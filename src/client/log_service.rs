use std::{collections::HashMap, time::Duration};

use hmac::{Hmac, Mac};
use md5::{Digest, Md5};
use reqwest::{
    header::{HeaderMap, HeaderValue},
    ClientBuilder, Response,
};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha1::Sha1;
use time::{macros::format_description, OffsetDateTime};

use crate::client::error::{Error, Result};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct LogServiceError {
    /// error code
    pub error_code: String,
    /// error message
    pub error_message: String,
}

/// Default const header.
const AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
const DEFAULT_HEADER: &[(&str, &str)] = &[
    ("x-log-apiversion", "0.6.0"),
    ("x-log-signaturemethod", "hmac-sha1"),
    ("user-agent", AGENT),
    ("x-sdk-client", AGENT),
];

type HamcSha1 = Hmac<Sha1>;

/// Config for request.
#[derive(Debug, Default)]
struct Request {
    method: String,
    uri: String,
    body: Option<String>,
    query: Vec<(String, String)>,
    headers: HeaderMap,
    project: Option<String>,
}

#[derive(Debug)]
pub struct LogServiceClient {
    /// The access key id of aliyun developer account.
    access_key_id: String,
    /// The access key secret of aliyun developer account.
    access_key_secret: String,
    /// The api endpoint of aliyun api service (need start with http:// or https://).
    endpoint: String,
    /// The http client builder used to send request.
    http_client_builder: ClientBuilder,
    /// The config of http request.
    request: Request,
}

impl LogServiceClient {
    /// Create a api client.
    pub fn new(
        access_key_id: impl Into<String>,
        access_key_secret: impl Into<String>,
        endpoint: impl Into<String>,
    ) -> Self {
        LogServiceClient {
            access_key_id: access_key_id.into(),
            access_key_secret: access_key_secret.into(),
            endpoint: endpoint.into(),
            http_client_builder: ClientBuilder::new(),
            request: Default::default(),
        }
    }

    /// Create a request with the `method` and `uri`.
    ///
    /// Returns a `Self` for send request.
    pub fn request(mut self, method: impl Into<String>, uri: impl Into<String>) -> Self {
        self.request.method = method.into();
        self.request.uri = uri.into();

        self
    }

    /// Create a `GET` request with the `uri`.
    ///
    /// Returns a `Self` for send request.
    pub fn get(self, uri: impl Into<String>) -> Self {
        self.request("GET".to_string(), uri.into())
    }

    /// Create a `POST` request with the `uri`.
    ///
    /// Returns a `Self` for send request.
    pub fn post(self, uri: impl Into<String>) -> Self {
        self.request("POST".to_string(), uri.into())
    }

    /// Set queries for request.
    ///
    /// Returns a `Self` for send request.
    pub fn query(mut self, queries: impl Into<Vec<(String, String)>>) -> Self {
        self.request.query = queries.into();

        self
    }

    /// Set body for request.
    ///
    /// Returns a `Self` for send request.
    pub fn body(mut self, body: impl Into<String>) -> Result<Self> {
        // compute body length and md5.
        let body = body.into();
        let mut hasher = Md5::new();
        hasher.update(body.as_bytes());
        let md5_result = hasher.finalize();

        // update headers.
        self.request
            .headers
            .insert("content-length", body.len().to_string().parse()?);
        self.request.headers.insert(
            "content-md5",
            base16ct::upper::encode_string(&md5_result).parse()?,
        );

        // store body string.
        self.request.body = Some(body);

        Ok(self)
    }

    /// Set header for request.
    ///
    /// Returns a `Self` for send request.
    pub fn header(mut self, headers: impl Into<HashMap<String, String>>) -> Result<Self> {
        self.request.headers = (&headers.into())
            .try_into()
            .map_err(|e| Error::InvalidRequest(format!("Cannot parse header: {}", e)))?;
        Ok(self)
    }

    /// Set project for request.
    ///
    /// Returns a `Self` for send request.
    pub fn project(mut self, project: impl Into<String>) -> Self {
        self.request.project = Some(project.into());

        self
    }

    /// Set a timeout for connect, read and write operations of a `Client`.
    ///
    /// Default is no timeout.
    pub fn timeout(mut self, timeout: Duration) -> Self {
        self.http_client_builder = self.http_client_builder.timeout(timeout);

        self
    }

    /// Send a request to service.
    /// Try to deserialize the response body as JSON.
    pub async fn json<T: DeserializeOwned>(self) -> Result<T> {
        Ok(self.send().await?.json::<T>().await?)
    }

    /// Send a request to service.
    /// Try to deserialize the response body as TEXT.
    pub async fn text(self) -> Result<String> {
        Ok(self.send().await?.text().await?)
    }

    /// Send a request to service.
    /// Return client Response.
    pub async fn send(mut self) -> Result<Response> {
        // check special header
        if !self.request.headers.contains_key("x-log-bodyrawsize") {
            self.request
                .headers
                .insert("x-log-bodyrawsize", "0".parse()?);
        }
        if !self.request.headers.contains_key("accept") {
            self.request
                .headers
                .insert("accept", "application/json".parse()?);
        }

        // add const header
        for (k, v) in DEFAULT_HEADER.iter() {
            self.request.headers.insert(*k, v.parse()?);
        }

        // add host header.
        let mut prefix = "";
        let mut host = self.endpoint.clone();
        if let Some(endpoint) = self.endpoint.strip_prefix("http://") {
            prefix = "http://";
            host = endpoint.to_string();
        } else if let Some(endpoint) = self.endpoint.strip_prefix("https://") {
            prefix = "https://";
            host = endpoint.to_string();
        }
        if let Some(project) = self.request.project.as_ref() {
            host = format!("{}.{}", project, host);
        }
        self.request.headers.insert("host", host.parse()?);

        // add date header.
        // RFC 1123: %a, %d %b %Y %H:%M:%S GMT
        let format = format_description!(
            "[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second] GMT"
        );
        let ts = OffsetDateTime::now_utc()
            .format(&format)
            .map_err(|e| Error::InvalidRequest(format!("Invalid RFC 1123 Date: {}", e)))?;
        self.request.headers.insert("date", ts.parse()?);

        // compute `Authorization` field.
        // Authorization = "SLS <AccessKeyId>:<Signature>"
        let authorization = format!("SLS {}:{}", self.access_key_id, self.signature()?);
        self.request
            .headers
            .insert("Authorization", authorization.parse()?);

        // build http client.
        let final_url = format!("{}{}{}", prefix, host, self.request.uri);
        let mut http_client = self.http_client_builder.build()?.request(
            self.request
                .method
                .parse()
                .map_err(|e| Error::InvalidRequest(format!("Invalid HTTP method: {}", e)))?,
            &final_url,
        );

        // set body.
        if let Some(body) = self.request.body {
            http_client = http_client.body(body);
        }

        // set query.
        if !self.request.query.is_empty() {
            http_client = http_client.query(&self.request.query);
        }

        // send request.
        let response = http_client.headers(self.request.headers).send().await?;

        // check HTTP StatusCode.
        if !response.status().is_success() {
            let result = response.json::<LogServiceError>().await?;
            return Err(Error::InvalidResponse {
                request_id: "".to_string(),
                error_code: result.error_code,
                error_message: result.error_message,
            });
        }

        // return response.
        Ok(response)
    }

    /// Compute canonicalized headers.
    fn canonicalized_headers(&self) -> String {
        let mut headers: Vec<(String, String)> = self
            .request
            .headers
            .iter()
            .filter_map(|(k, v)| {
                let k = k.as_str().to_lowercase();
                if k.starts_with("x-acs-") || k.starts_with("x-log-") {
                    Some((k, v.to_str().unwrap().to_string()))
                } else {
                    None
                }
            })
            .collect();
        headers.sort_by(|a, b| a.0.cmp(&b.0));

        let headers: Vec<String> = headers
            .iter()
            .map(|(k, v)| format!("{}:{}", k, v))
            .collect();

        headers.join("\n")
    }

    /// Compute canonicalized resource.
    fn canonicalized_resource(&self) -> String {
        if !self.request.query.is_empty() {
            let mut params = self.request.query.clone();
            params.sort_by_key(|item| item.0.clone());
            let params: Vec<String> = params.iter().map(|(k, v)| format!("{}={}", k, v)).collect();
            let sorted_query_string = params.join("&");
            format!("{}?{}", self.request.uri, sorted_query_string)
        } else {
            self.request.uri.clone()
        }
    }

    /// Compute signature for request.
    fn signature(&self) -> Result<String> {
        // build body.
        let canonicalized_headers = self.canonicalized_headers();
        let canonicalized_resource = self.canonicalized_resource();
        let body = format!(
            "{}\n{}\n{}\n{}\n{}\n{}",
            self.request.method.to_uppercase(),
            self.request
                .headers
                .get("content-md5")
                .unwrap_or(&HeaderValue::from_static(""))
                .to_str()
                .unwrap(),
            self.request
                .headers
                .get("content-type")
                .unwrap_or(&HeaderValue::from_static(""))
                .to_str()
                .unwrap(),
            self.request.headers["date"].to_str().unwrap(),
            canonicalized_headers,
            canonicalized_resource
        );

        // sign body.
        let mut mac = HamcSha1::new_from_slice(self.access_key_secret.as_bytes())
            .map_err(|e| Error::InvalidRequest(format!("Invalid HMAC-SHA1 secret key: {}", e)))?;
        mac.update(body.as_bytes());
        let result = mac.finalize();
        let code = result.into_bytes();

        Ok(base64::encode(code))
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::env;

    #[tokio::test]
    async fn get_log_store_test() -> Result<()> {
        let client = LogServiceClient::new(
            env::var("ACCESS_KEY_ID").unwrap(),
            env::var("ACCESS_KEY_SECRET").unwrap(),
            "https://cn-hangzhou.log.aliyuncs.com",
        );

        let err = client
            .get(format!("/logstores/{}", "logstore"))
            .project("project")
            .text()
            .await
            .unwrap_err();

        // send request
        if let Error::InvalidResponse {
            request_id: _,
            error_code,
            error_message: _,
        } = err
        {
            assert_eq!(error_code, "Unauthorized");
        } else {
            return Err(err);
        }

        Ok(())
    }
}
