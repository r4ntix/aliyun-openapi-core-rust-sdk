use std::{collections::HashMap, time::Duration};

use hmac::{Hmac, Mac};
use reqwest::{header::HeaderMap, ClientBuilder, Response};
use serde::{de::DeserializeOwned, Deserialize, Serialize};
use sha1::Sha1;
use time::{format_description::well_known::Iso8601, OffsetDateTime};
use url::form_urlencoded::byte_serialize;

use crate::client::error::{Error, Result};

#[derive(Debug, Deserialize, Serialize)]
#[serde(rename_all = "PascalCase")]
pub struct RPCServiceError {
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
const DEFAULT_HEADER: &[(&str, &str)] = &[("user-agent", AGENT), ("x-sdk-client", AGENT)];
const DEFAULT_PARAM: &[(&str, &str)] = &[
    ("Format", "JSON"),
    ("SignatureMethod", "HMAC-SHA1"),
    ("SignatureVersion", "1.0"),
];

type HamcSha1 = Hmac<Sha1>;

/// Config for request.
#[derive(Debug, Default)]
struct Request {
    action: String,
    method: String,
    query: Vec<(String, String)>,
    headers: HeaderMap,
    version: String,
}

#[derive(Debug)]
pub struct RPClient {
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

impl RPClient {
    /// Create a api client.
    pub fn new(
        access_key_id: impl Into<String>,
        access_key_secret: impl Into<String>,
        endpoint: impl Into<String>,
    ) -> Self {
        RPClient {
            access_key_id: access_key_id.into(),
            access_key_secret: access_key_secret.into(),
            endpoint: endpoint.into(),
            http_client_builder: ClientBuilder::new(),
            request: Default::default(),
        }
    }

    /// Create a request with the `method` and `action`.
    ///
    /// Returns a `Self` for send request.
    pub fn request(mut self, method: impl Into<String>, action: impl Into<String>) -> Self {
        self.request.method = method.into();
        self.request.action = action.into();

        self
    }

    /// Create a `GET` request with the `action`.
    ///
    /// Returns a `Self` for send request.
    pub fn get(self, action: impl Into<String>) -> Self {
        self.request("GET".to_string(), action.into())
    }

    /// Create a `POST` request with the `action`.
    ///
    /// Returns a `Self` for send request.
    pub fn post(self, action: impl Into<String>) -> Self {
        self.request("POST".to_string(), action.into())
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

    /// Set header for request.
    ///
    /// Returns a `Self` for send request.
    pub fn header(mut self, headers: impl Into<HashMap<String, String>>) -> Result<Self> {
        self.request.headers = (&headers.into())
            .try_into()
            .map_err(|e| Error::InvalidRequest(format!("Cannot parse header: {e}")))?;
        Ok(self)
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
        // add const header
        for (k, v) in DEFAULT_HEADER.iter() {
            self.request.headers.insert(*k, v.parse()?);
        }

        // build params.
        let now_utc = OffsetDateTime::now_utc();
        let nonce = now_utc.unix_timestamp_nanos().to_string();
        let ts = now_utc
            .format(&Iso8601::DEFAULT)
            .map_err(|e| Error::InvalidRequest(format!("Invalid ISO 8601 Date: {e}")))?;

        let mut params = Vec::from(DEFAULT_PARAM);
        params.push(("Action", &self.request.action));
        params.push(("AccessKeyId", &self.access_key_id));
        params.push(("SignatureNonce", &nonce));
        params.push(("Timestamp", &ts));
        params.push(("Version", &self.request.version));
        params.extend(
            self.request
                .query
                .iter()
                .map(|(k, v)| (k.as_ref(), v.as_ref())),
        );
        params.sort_by_key(|item| item.0);

        // encode params.
        let params: Vec<String> = params
            .into_iter()
            .map(|(k, v)| format!("{}={}", url_encode(k), url_encode(v)))
            .collect();
        let sorted_query_string = params.join("&");
        let string_to_sign = format!(
            "GET&{}&{}",
            url_encode("/"),
            url_encode(&sorted_query_string)
        );

        // sign params, get finnal request url.
        let sign = sign(&format!("{}&", self.access_key_secret), &string_to_sign)?;
        let signature = url_encode(&sign);
        let final_url = format!(
            "{}?Signature={}&{}",
            self.endpoint, signature, sorted_query_string
        );

        // send request.
        let http_client = self.http_client_builder.build()?.request(
            self.request
                .method
                .parse()
                .map_err(|e| Error::InvalidRequest(format!("Invalid HTTP method: {}", e)))?,
            &final_url,
        );
        let response = http_client.headers(self.request.headers).send().await?;

        // check HTTP StatusCode.
        if !response.status().is_success() {
            let result = response.json::<RPCServiceError>().await?;
            return Err(Error::InvalidResponse {
                request_id: result.request_id,
                error_code: result.code,
                error_message: result.message,
            });
        }

        // return response.
        Ok(response)
    }
}

fn sign(key: &str, body: &str) -> Result<String> {
    let mut mac = HamcSha1::new_from_slice(key.as_bytes())
        .map_err(|e| Error::InvalidRequest(format!("Invalid HMAC-SHA1 secret key: {}", e)))?;
    mac.update(body.as_bytes());
    let result = mac.finalize();
    let code = result.into_bytes();

    Ok(base64::encode(code))
}

/// URL encode following [RFC3986](https://www.rfc-editor.org/rfc/rfc3986)
fn url_encode(s: &str) -> String {
    let s: String = byte_serialize(s.as_bytes()).collect();
    s.replace("+", "%20")
        .replace("*", "%2A")
        .replace("%7E", "~")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn url_encode_test() -> Result<()> {
        assert_eq!(
            url_encode("begin_+_*_~_-_._\"_ end"),
            "begin_%2B_%2A_~_-_._%22_%20end"
        );

        Ok(())
    }

    #[tokio::test]
    async fn rpc_client_invalid_access_key_id_test() -> Result<()> {
        // create rpc style api client.
        let aliyun_openapi_client = RPClient::new(
            "access_key_id",
            "access_key_secret",
            "https://ecs-cn-hangzhou.aliyuncs.com",
        );

        // call `DescribeRegions` with empty queries.
        match aliyun_openapi_client
            .version("2014-05-26")
            .get("DescribeRegions")
            .text()
            .await
            .unwrap_err()
        {
            Error::InvalidResponse { error_code, .. } => {
                assert_eq!(error_code, "InvalidAccessKeyId.NotFound")
            }
            _ => assert!(false),
        };

        Ok(())
    }

    #[tokio::test]
    async fn rpc_client_get_with_query_test() -> Result<()> {
        // create rpc style api client.
        let aliyun_openapi_client = RPClient::new(
            "access_key_id",
            "access_key_secret",
            "https://ecs-cn-hangzhou.aliyuncs.com",
        );

        match aliyun_openapi_client
            .version("2014-05-26")
            .get("DescribeInstances")
            .query(vec![("RegionId", "cn-hangzhou")])
            .text()
            .await
            .unwrap_err()
        {
            Error::InvalidResponse { error_code, .. } => {
                assert_eq!(error_code, "InvalidAccessKeyId.NotFound")
            }
            _ => assert!(false),
        };

        Ok(())
    }
}
