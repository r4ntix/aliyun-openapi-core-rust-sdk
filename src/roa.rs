use anyhow::{anyhow, Result};
use hmac::{Hmac, Mac};
use md5::{Digest, Md5};
use reqwest::blocking::ClientBuilder;
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use sha1::Sha1;
use std::env;
use std::time::Duration;
use std::{borrow::Borrow, str::FromStr};
use time::macros::format_description;
use time::OffsetDateTime;
use url::Url;
use uuid::Uuid;

/// Default const header.
const DEFAULT_HEADER: &[(&str, &str)] = &[
    ("accept", "application/json"),
    ("x-acs-signature-method", "HMAC-SHA1"),
    ("x-acs-signature-version", "1.0"),
];

type HamcSha1 = Hmac<Sha1>;

/// Config for request.
#[derive(Debug)]
struct Request {
    method: String,
    uri: String,
    body: Option<String>,
    query: Vec<(String, String)>,
    headers: HeaderMap,
}

/// The roa style api client.
#[deprecated(
    since = "1.0.0",
    note = "Please use the `aliyun_openapi_core_rust_sdk::client::roa::ROAClient` instead"
)]
#[derive(Clone, Debug)]
pub struct Client {
    /// The access key id of aliyun developer account.
    access_key_id: String,
    /// The access key secret of aliyun developer account.
    access_key_secret: String,
    /// The api endpoint of aliyun api service (need start with http:// or https://).
    endpoint: String,
    /// The api version of aliyun api service.
    version: String,
}

impl Client {
    #![allow(deprecated)]

    /// Create a roa style api client.
    pub fn new(
        access_key_id: String,
        access_key_secret: String,
        endpoint: String,
        version: String,
    ) -> Self {
        Client {
            access_key_id,
            access_key_secret,
            endpoint,
            version,
        }
    }

    /// Create a request with the `method` and `uri`.
    ///
    /// Returns a `RequestBuilder` for send request.
    pub fn execute(&self, method: &str, uri: &str) -> RequestBuilder {
        RequestBuilder::new(
            &self.access_key_id,
            &self.access_key_secret,
            &self.endpoint,
            &self.version,
            String::from(method),
            String::from(uri),
        )
    }

    /// Create a `GET` request with the `uri`.
    ///
    /// Returns a `RequestBuilder` for send request.
    pub fn get(&self, uri: &str) -> RequestBuilder {
        self.execute("GET", uri)
    }

    /// Create a `POST` request with the `uri`.
    ///
    /// Returns a `RequestBuilder` for send request.
    pub fn post(&self, uri: &str) -> RequestBuilder {
        self.execute("POST", uri)
    }

    /// Create a `PUT` request with the `uri`.
    ///
    /// Returns a `RequestBuilder` for send request.
    pub fn put(&self, uri: &str) -> RequestBuilder {
        self.execute("PUT", uri)
    }
}

/// The request builder struct.
#[derive(Debug)]
pub struct RequestBuilder<'a> {
    /// The access key id of aliyun developer account.
    access_key_id: &'a str,
    /// The access key secret of aliyun developer account.
    access_key_secret: &'a str,
    /// The api endpoint of aliyun api service (need start with http:// or https://).
    endpoint: &'a str,
    /// The http client builder used to send request.
    http_client_builder: ClientBuilder,
    /// The config of http request.
    request: Request,
}

impl<'a> RequestBuilder<'a> {
    /// Create a request object.
    pub fn new(
        access_key_id: &'a str,
        access_key_secret: &'a str,
        endpoint: &'a str,
        version: &'a str,
        method: String,
        uri: String,
    ) -> Self {
        // init http headers.
        let mut headers = HeaderMap::new();
        for (k, v) in DEFAULT_HEADER.iter() {
            headers.insert(*k, v.parse().unwrap());
        }
        headers.insert(
            "user-agent",
            format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
                .parse()
                .unwrap(),
        );
        headers.insert(
            "x-sdk-client",
            format!("{}/{}", env!("CARGO_PKG_NAME"), env!("CARGO_PKG_VERSION"))
                .parse()
                .unwrap(),
        );
        headers.insert("x-acs-version", version.parse().unwrap());

        // return RequestBuilder.
        RequestBuilder {
            access_key_id,
            access_key_secret,
            endpoint,
            http_client_builder: ClientBuilder::new(),
            request: Request {
                method,
                uri,
                body: None,
                query: Vec::new(),
                headers,
            },
        }
    }

    /// Set body for request.
    pub fn body(mut self, body: &str) -> Result<Self> {
        // compute body length and md5.
        let body = body.to_string();
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
    pub fn header<I>(mut self, iter: I) -> Self
    where
        I: IntoIterator,
        I::Item: Borrow<(&'a str, &'a str)>,
    {
        for i in iter.into_iter() {
            let h = i.borrow();
            let key = HeaderName::from_str(h.0);
            let value = HeaderValue::from_str(h.1);
            // ingore invailid header.
            if let Ok(key) = key {
                if let Ok(value) = value {
                    self.request.headers.insert(key, value);
                }
            }
        }
        self
    }

    /// Set queries for request.
    pub fn query<I>(mut self, iter: I) -> Self
    where
        I: IntoIterator,
        I::Item: Borrow<(&'a str, &'a str)>,
    {
        for i in iter.into_iter() {
            let b = i.borrow();
            self.request.query.push((b.0.to_string(), b.1.to_string()));
        }
        self
    }

    /// Send a request to api service.
    pub fn send(mut self) -> Result<String> {
        // add date header.
        // RFC 1123: %a, %d %b %Y %H:%M:%S GMT
        let format = format_description!(
            "[weekday repr:short], [day] [month repr:short] [year] [hour]:[minute]:[second] GMT"
        );
        let ts = OffsetDateTime::now_utc()
            .format(&format)
            .map_err(|e| anyhow!(format!("Invalid RFC 1123 Date: {}", e)))?;
        self.request.headers.insert("date", ts.parse()?);

        // add nonce header.
        let nonce = Uuid::new_v4().to_string();
        self.request
            .headers
            .insert("x-acs-signature-nonce", nonce.parse()?);

        // parse host of self.endpoint.
        let endpoint = Url::parse(self.endpoint)?;
        let host = endpoint
            .host_str()
            .ok_or_else(|| anyhow!("parse endpoint failed"))?;
        self.request.headers.insert("host", host.parse()?);

        // compute `Authorization` field.
        let authorization = format!("acs {}:{}", self.access_key_id, self.signature()?);
        self.request
            .headers
            .insert("Authorization", authorization.parse()?);

        // build http client.
        let final_url = format!("{}{}", self.endpoint, self.request.uri);
        let mut http_client = self
            .http_client_builder
            .build()?
            .request(self.request.method.parse()?, final_url);

        // set body.
        if let Some(body) = self.request.body {
            http_client = http_client.body(body);
        }

        // send request.
        let response = http_client
            .headers(self.request.headers)
            .query(&self.request.query)
            .send()?
            .text()?;

        // return response.
        Ok(response)
    }

    /// Set a timeout for connect, read and write operations of a `Client`.
    ///
    /// Default is 30 seconds.
    ///
    /// Pass `None` to disable timeout.
    pub fn timeout<T>(mut self, timeout: T) -> Self
    where
        T: Into<Option<Duration>>,
    {
        self.http_client_builder = self.http_client_builder.timeout(timeout);
        self
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
            .map_err(|e| anyhow!(format!("Invalid HMAC-SHA1 secret key: {}", e)))?;
        mac.update(body.as_bytes());
        let result = mac.finalize();
        let code = result.into_bytes();

        Ok(base64::encode(code))
    }
}

#[cfg(test)]
mod tests {
    #![allow(deprecated)]

    use std::collections::HashMap;

    use serde_json::json;

    use super::*;

    #[test]
    fn roa_client_get_no_query() -> Result<()> {
        // create roa style api client.
        let aliyun_openapi_client = Client::new(
            env::var("ACCESS_KEY_ID")?,
            env::var("ACCESS_KEY_SECRET")?,
            String::from("https://ros.aliyuncs.com"),
            String::from("2015-09-01"),
        );

        // call `DescribeRegions` with empty queries.
        let response = aliyun_openapi_client.get("/regions").send()?;

        assert!(response.contains("Regions"));

        Ok(())
    }

    #[test]
    fn roa_client_get_with_timeout() -> Result<()> {
        // create roa style api client.
        let aliyun_openapi_client = Client::new(
            env::var("ACCESS_KEY_ID")?,
            env::var("ACCESS_KEY_SECRET")?,
            String::from("https://ros.aliyuncs.com"),
            String::from("2015-09-01"),
        );

        // call `DescribeRegions` with empty queries.
        let response = aliyun_openapi_client
            .get("/regions")
            .timeout(Duration::from_millis(1))
            .send();

        assert!(response.is_err());

        Ok(())
    }

    #[test]
    fn roa_client_post_with_json_params() -> Result<()> {
        // create roa style api client.
        let aliyun_openapi_client = Client::new(
            env::var("ACCESS_KEY_ID")?,
            env::var("ACCESS_KEY_SECRET")?,
            String::from("http://mt.aliyuncs.com"),
            String::from("2019-01-02"),
        );

        // create params.
        let mut params = HashMap::new();
        params.insert("SourceText", "你好");
        params.insert("SourceLanguage", "zh");
        params.insert("TargetLanguage", "en");
        params.insert("FormatType", "text");
        params.insert("Scene", "general");

        // call `DescribeRegions` with empty queries.
        let response = aliyun_openapi_client
            .post("/api/translate/web/general")
            .header(&[("Content-Type", "application/json")])
            .body(&json!(params).to_string())?
            .send()?;

        assert!(response.contains("Hello"));

        Ok(())
    }
}
