use anyhow::Result;
use chrono::{Local, Utc};
use crypto::{hmac::Hmac, mac::Mac, sha1::Sha1};
use reqwest::ClientBuilder;
use std::borrow::Borrow;
use std::time::Duration;
use url::form_urlencoded::byte_serialize;

/// Default const param.
const DEFAULT_PARAM: &[(&str, &str)] = &[
    ("Format", "JSON"),
    ("SignatureMethod", "HMAC-SHA1"),
    ("SignatureVersion", "1.0"),
];

/// Config for request.
#[derive(Debug)]
struct Request {
    action: String,
    method: String,
    query: Vec<(String, String)>,
}

/// The rpc style api client.
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
        // gen timestamp.
        let nonce = Local::now().timestamp_subsec_nanos().to_string();
        let ts = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        // build params.
        let mut params = Vec::from(DEFAULT_PARAM);
        params.push(("Action", &action));
        params.push(("AccessKeyId", &self.access_key_id));
        params.push(("SignatureNonce", &nonce));
        params.push(("Timestamp", &ts));
        params.push(("Version", &self.version));
        params.extend_from_slice(&queries);
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
        let sign = sign(&format!("{}&", self.access_key_secret), &string_to_sign);
        let signature = url_encode(&sign);
        let final_url = format!(
            "{}?Signature={}&{}",
            self.endpoint, signature, sorted_query_string
        );

        // send request.
        let response = reqwest::get(&final_url)?.text()?;

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
        // gen timestamp.
        let nonce = Local::now().timestamp_subsec_nanos().to_string();
        let ts = Utc::now().format("%Y-%m-%dT%H:%M:%SZ").to_string();

        // build params.
        let mut params = Vec::from(DEFAULT_PARAM);
        params.push(("Action", &self.request.action));
        params.push(("AccessKeyId", &self.access_key_id));
        params.push(("SignatureNonce", &nonce));
        params.push(("Timestamp", &ts));
        params.push(("Version", &self.version));
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
        let sign = sign(&format!("{}&", self.access_key_secret), &string_to_sign);
        let signature = url_encode(&sign);
        let final_url = format!(
            "{}?Signature={}&{}",
            self.endpoint, signature, sorted_query_string
        );

        // build http client.
        let http_client = self
            .http_client_builder
            .build()?
            .request(self.request.method.parse()?, &final_url);

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

fn sign(key: &str, body: &str) -> String {
    let mut mac = Hmac::new(Sha1::new(), key.as_bytes());
    mac.input(body.as_bytes());
    let result = mac.result();
    let code = result.code();
    base64::encode(code)
}

fn url_encode(s: &str) -> String {
    let s: String = byte_serialize(s.as_bytes()).collect();
    s.replace("+", "%20")
        .replace("*", "%2A")
        .replace("%7E", "~")
}
