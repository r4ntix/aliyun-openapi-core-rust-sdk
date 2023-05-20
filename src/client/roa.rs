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
use url::Url;

use crate::client::error::{Error, Result};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct ROAServiceError {
    /// Error code
    pub code: String,
    /// Error message
    pub message: String,
    /// Request id
    #[serde(default)]
    pub request_id: String,
    /// Recommend
    #[serde(default)]
    pub recommend: String,
}

/// Default const header.
const AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
const DEFAULT_HEADER: &[(&str, &str)] = &[
    ("accept", "application/json"),
    ("x-acs-signature-method", "HMAC-SHA1"),
    ("user-agent", AGENT),
    ("x-sdk-client", AGENT),
];

type HamcSha1 = Hmac<Sha1>;

/// Config for request.
#[derive(Clone, Debug, Default)]
struct Request {
    method: String,
    uri: String,
    body: Option<String>,
    query: Vec<(String, String)>,
    headers: HeaderMap,
    project: Option<String>,
    version: String,
    timeout: Option<Duration>,
}

#[derive(Clone, Debug)]
pub struct ROAClient {
    /// The access key id of aliyun developer account.
    access_key_id: String,
    /// The access key secret of aliyun developer account.
    access_key_secret: String,
    /// The api endpoint of aliyun api service (need start with http:// or https://).
    endpoint: String,
    /// The config of http request.
    request: Request,
}

impl ROAClient {
    /// Create a api client.
    pub fn new(
        access_key_id: impl Into<String>,
        access_key_secret: impl Into<String>,
        endpoint: impl Into<String>,
    ) -> Self {
        ROAClient {
            access_key_id: access_key_id.into(),
            access_key_secret: access_key_secret.into(),
            endpoint: endpoint.into(),
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
    pub fn query<I, T>(mut self, queries: I) -> Self
    where
        I: IntoIterator<Item = (T, T)>,
        T: Into<String>,
    {
        self.request.query = queries
            .into_iter()
            .map(|v| (v.0.into(), v.1.into()))
            .collect();

        self
    }

    /// Set version for request.
    ///
    /// Returns a `Self` for send request.
    pub fn version(mut self, version: impl Into<String>) -> Self {
        self.request.version = version.into();

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
        self.request
            .headers
            .insert("content-md5", base64::encode(md5_result).parse()?);

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
        self.request.timeout = Some(timeout);

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
        // add const header
        for (k, v) in DEFAULT_HEADER.iter() {
            self.request.headers.insert(*k, v.parse()?);
        }

        // add host header.
        let endpoint = Url::parse(&self.endpoint)
            .map_err(|e| Error::InvalidRequest(format!("Invalid endpoint: {e}")))?;
        let host = endpoint
            .host_str()
            .ok_or_else(|| Error::InvalidRequest(format!("Invalid endpoint: {endpoint}")))?;
        self.request.headers.insert("host", host.parse()?);

        // add date header.
        // RFC 1123: %a, %d %b %Y %H:%M:%S GMT
        let format = format_description!(
            "[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second] GMT"
        );
        let now_utc = OffsetDateTime::now_utc();
        let ts = now_utc
            .format(&format)
            .map_err(|e| Error::InvalidRequest(format!("Invalid RFC 1123 Date: {}", e)))?;
        self.request.headers.insert("date", ts.parse()?);

        // add nonce header.
        let nonce = now_utc.unix_timestamp_nanos().to_string();
        self.request
            .headers
            .insert("x-acs-signature-nonce", nonce.parse()?);

        // compute `Authorization` field.
        // Authorization = "acs <AccessKeyId>:<Signature>"
        let authorization = format!("acs {}:{}", self.access_key_id, self.signature()?);
        self.request
            .headers
            .insert("Authorization", authorization.parse()?);

        // build http client.
        let final_url = format!("{}{}", self.endpoint, self.request.uri);
        let mut http_client_builder = ClientBuilder::new();
        if let Some(timeout) = self.request.timeout {
            http_client_builder = http_client_builder.timeout(timeout);
        }
        let mut http_client = http_client_builder.build()?.request(
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
            let result = response.json::<ROAServiceError>().await?;
            return Err(Error::InvalidResponse {
                request_id: result.request_id,
                error_code: result.code,
                error_message: result.message,
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
                if k.starts_with("x-acs-") {
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
            "{}\n{}\n{}\n{}\n{}\n{}\n{}",
            self.request.method.to_uppercase(),
            self.request.headers["accept"].to_str().unwrap(),
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
    use std::env;

    use serde_json::json;

    use super::*;

    #[tokio::test]
    async fn roa_client_invalid_access_key_id_test() -> Result<()> {
        // create roa style api client.
        let aliyun_openapi_client = ROAClient::new(
            env::var("ACCESS_KEY_ID").unwrap(),
            env::var("ACCESS_KEY_SECRET").unwrap(),
            "https://ros.aliyuncs.com",
        );

        // call `DescribeRegions` with empty queries.
        let response = aliyun_openapi_client
            .version("2015-09-01")
            .get("/regions")
            .text()
            .await?;

        assert!(response.contains("Regions"));

        Ok(())
    }

    #[tokio::test]
    async fn roa_client_get_with_timeout() -> Result<()> {
        // create roa style api client.
        let aliyun_openapi_client = ROAClient::new(
            env::var("ACCESS_KEY_ID").unwrap(),
            env::var("ACCESS_KEY_SECRET").unwrap(),
            "https://ros.aliyuncs.com",
        );

        // call `DescribeRegions` with empty queries.
        let response = aliyun_openapi_client
            .version("2015-09-01")
            .get("/regions")
            .timeout(Duration::from_millis(1))
            .text()
            .await;

        assert!(response.is_err());

        Ok(())
    }

    #[tokio::test]
    async fn roa_client_get_with_query_test() -> Result<()> {
        // create roa style api client.
        let aliyun_openapi_client = ROAClient::new(
            env::var("ACCESS_KEY_ID").unwrap(),
            env::var("ACCESS_KEY_SECRET").unwrap(),
            "http://mt.aliyuncs.com",
        );

        // create params.
        let mut params = HashMap::new();
        params.insert("SourceText", "你好");
        params.insert("SourceLanguage", "zh");
        params.insert("TargetLanguage", "en");
        params.insert("FormatType", "text");
        params.insert("Scene", "general");

        let response = aliyun_openapi_client
            .version("2018-04-08")
            .post("/api/translate/web/general")
            .header([("Content-Type".to_string(), "application/json".to_string())])?
            .body(json!(params).to_string())?
            .text()
            .await?;

        assert!(response.contains("Hello"));

        Ok(())
    }
}
