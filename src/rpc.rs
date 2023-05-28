use anyhow::{anyhow, Result};
use hmac::{Hmac, Mac};
use reqwest::blocking::ClientBuilder;
use sha1::Sha1;
use std::borrow::Borrow;
use std::time::Duration;
use time::format_description::well_known::Iso8601;
use time::OffsetDateTime;
use url::form_urlencoded::byte_serialize;
use uuid::Uuid;

/// Default const param.
const DEFAULT_PARAM: &[(&str, &str)] = &[
    ("Format", "JSON"),
    ("SignatureMethod", "HMAC-SHA1"),
    ("SignatureVersion", "1.0"),
];

type HamcSha1 = Hmac<Sha1>;

/// Config for request.
#[derive(Debug)]
struct Request {
    action: String,
    method: String,
    query: Vec<(String, String)>,
}

/// The rpc style api client.
#[deprecated(
    since = "1.0.0",
    note = "Please use the `aliyun_openapi_core_rust_sdk::client::rpc::RPClient` instead"
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

    /// Create a rpc style api client.
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

    /// Create a `GET` request with the `action`.
    ///
    /// Returns a `RequestBuilder` for send request.
    pub fn get(&self, action: &str) -> RequestBuilder {
        self.execute("GET", action)
    }

    /// Create a `POST` request with the `action`.
    ///
    /// Returns a `RequestBuilder` for send request.
    pub fn post(&self, action: &str) -> RequestBuilder {
        self.execute("POST", action)
    }

    /// Create a request with the `method` and `action`.
    fn execute(&self, method: &str, action: &str) -> RequestBuilder {
        RequestBuilder::new(
            &self.access_key_id,
            &self.access_key_secret,
            &self.endpoint,
            &self.version,
            String::from(method),
            String::from(action),
        )
    }

    /// Send a request to api service.
    ///
    /// if queries is empty, can pass `&[]`
    #[deprecated(since = "0.3.0", note = "Please use the `get` function instead")]
    pub fn request(&self, action: &str, queries: &[(&str, &str)]) -> Result<String> {
        // build params.
        let nonce = Uuid::new_v4().to_string();
        let ts = OffsetDateTime::now_utc()
            .format(&Iso8601::DEFAULT)
            .map_err(|e| anyhow!(format!("Invalid ISO 8601 Date: {e}")))?;

        let mut params = Vec::from(DEFAULT_PARAM);
        params.push(("Action", action));
        params.push(("AccessKeyId", &self.access_key_id));
        params.push(("SignatureNonce", &nonce));
        params.push(("Timestamp", &ts));
        params.push(("Version", &self.version));
        params.extend_from_slice(queries);
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
        let response = reqwest::blocking::get(final_url)?.text()?;

        // return response.
        Ok(response)
    }
}

/// The RequestBuilder.
pub struct RequestBuilder<'a> {
    /// The access key id of aliyun developer account.
    access_key_id: &'a str,
    /// The access key secret of aliyun developer account.
    access_key_secret: &'a str,
    /// The api endpoint of aliyun api service (need start with http:// or https://).
    endpoint: &'a str,
    /// The api version of aliyun api service.
    version: &'a str,
    /// The config of http request.
    request: Request,
    /// The http client builder used to send request.
    http_client_builder: ClientBuilder,
}

impl<'a> RequestBuilder<'a> {
    /// Create a request object.
    pub fn new(
        access_key_id: &'a str,
        access_key_secret: &'a str,
        endpoint: &'a str,
        version: &'a str,
        method: String,
        action: String,
    ) -> Self {
        RequestBuilder {
            access_key_id,
            access_key_secret,
            endpoint,
            version,
            request: Request {
                action,
                method,
                query: Vec::new(),
            },
            http_client_builder: ClientBuilder::new(),
        }
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
    pub fn send(self) -> Result<String> {
        // build params.
        let nonce = Uuid::new_v4().to_string();
        let ts = OffsetDateTime::now_utc()
            .format(&Iso8601::DEFAULT)
            .map_err(|e| anyhow!(format!("Invalid ISO 8601 Date: {e}")))?;

        let mut params = Vec::from(DEFAULT_PARAM);
        params.push(("Action", &self.request.action));
        params.push(("AccessKeyId", self.access_key_id));
        params.push(("SignatureNonce", &nonce));
        params.push(("Timestamp", &ts));
        params.push(("Version", self.version));
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
            "{}&{}&{}",
            self.request.method,
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

        // build http client.
        let http_client = self
            .http_client_builder
            .build()?
            .request(self.request.method.parse()?, final_url);

        // send request.
        let response = http_client.send()?.text()?;

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
}

fn sign(key: &str, body: &str) -> Result<String> {
    let mut mac = HamcSha1::new_from_slice(key.as_bytes())
        .map_err(|e| anyhow!(format!("Invalid HMAC-SHA1 secret key: {}", e)))?;
    mac.update(body.as_bytes());
    let result = mac.finalize();
    let code = result.into_bytes();

    Ok(base64::encode(code))
}

fn url_encode(s: &str) -> String {
    let s: String = byte_serialize(s.as_bytes()).collect();
    s.replace('+', "%20")
        .replace('*', "%2A")
        .replace("%7E", "~")
}

#[cfg(test)]
mod tests {
    #![allow(deprecated)]

    use std::env;

    use super::*;

    // 0.2.0 version, rpc style client test.
    #[test]
    fn rpc_client_no_query_compatibility_020() -> Result<()> {
        // create rpc style api client.
        let aliyun_openapi_client = Client::new(
            env::var("ACCESS_KEY_ID")?,
            env::var("ACCESS_KEY_SECRET")?,
            String::from("https://ecs.aliyuncs.com/"),
            String::from("2014-05-26"),
        );

        // call `DescribeRegions` with empty queries.
        let response = aliyun_openapi_client.request("DescribeRegions", &[])?;

        assert!(response.contains("Regions"));

        Ok(())
    }

    // 0.2.0 version, rpc style client test with query.
    #[test]
    fn rpc_client_with_query_compatibility_020() -> Result<()> {
        // create rpc style api client.
        let aliyun_openapi_client = Client::new(
            env::var("ACCESS_KEY_ID")?,
            env::var("ACCESS_KEY_SECRET")?,
            String::from("https://ecs.aliyuncs.com/"),
            String::from("2014-05-26"),
        );

        // call `DescribeInstances` with queries.
        let response =
            aliyun_openapi_client.request("DescribeInstances", &[("RegionId", "cn-hangzhou")])?;

        assert!(response.contains("Instances"));

        Ok(())
    }

    // rpc style client `GET` test.
    #[test]
    fn rpc_client_get_no_query() -> Result<()> {
        // create rpc style api client.
        let aliyun_openapi_client = Client::new(
            env::var("ACCESS_KEY_ID")?,
            env::var("ACCESS_KEY_SECRET")?,
            String::from("https://ecs.aliyuncs.com/"),
            String::from("2014-05-26"),
        );

        // call `DescribeRegions` with empty queries.
        let response = aliyun_openapi_client.get("DescribeRegions").send()?;

        assert!(response.contains("Regions"));

        Ok(())
    }

    // rpc style client `GET` test with query.
    #[test]
    fn rpc_client_get_with_query() -> Result<()> {
        // create rpc style api client.
        let aliyun_openapi_client = Client::new(
            env::var("ACCESS_KEY_ID")?,
            env::var("ACCESS_KEY_SECRET")?,
            String::from("https://ecs.aliyuncs.com/"),
            String::from("2014-05-26"),
        );

        // call `DescribeInstances` with queries.
        let response = aliyun_openapi_client
            .get("DescribeInstances")
            .query(&[("RegionId", "cn-hangzhou")])
            .send()?;

        assert!(response.contains("Instances"));

        Ok(())
    }

    // rpc style client `GET` test with timeout.
    #[test]
    fn rpc_client_get_with_timeout() -> Result<()> {
        // create rpc style api client.
        let aliyun_openapi_client = Client::new(
            env::var("ACCESS_KEY_ID")?,
            env::var("ACCESS_KEY_SECRET")?,
            String::from("https://ecs.aliyuncs.com/"),
            String::from("2014-05-26"),
        );

        // call `DescribeRegions` with empty queries.
        let response = aliyun_openapi_client
            .get("DescribeRegions")
            .timeout(Duration::from_millis(1))
            .send();

        assert!(response.is_err());

        Ok(())
    }

    // rpc style client `POST` test.
    #[test]
    fn rpc_client_post_no_query() -> Result<()> {
        // create rpc style api client.
        let aliyun_openapi_client = Client::new(
            env::var("ACCESS_KEY_ID")?,
            env::var("ACCESS_KEY_SECRET")?,
            String::from("https://ecs.aliyuncs.com/"),
            String::from("2014-05-26"),
        );

        // call `DescribeRegions` with empty queries.
        let response = aliyun_openapi_client.post("DescribeRegions").send()?;

        assert!(response.contains("Regions"));

        Ok(())
    }

    // rpc style client `POST` test with query.
    #[test]
    fn rpc_client_post_with_query() -> Result<()> {
        // create rpc style api client.
        let aliyun_openapi_client = Client::new(
            env::var("ACCESS_KEY_ID")?,
            env::var("ACCESS_KEY_SECRET")?,
            String::from("https://ecs.aliyuncs.com/"),
            String::from("2014-05-26"),
        );

        // call `DescribeInstances` with queries.
        let response = aliyun_openapi_client
            .post("DescribeInstances")
            .query(&[("RegionId", "cn-hangzhou")])
            .send()?;

        assert!(response.contains("Instances"));

        Ok(())
    }
}
