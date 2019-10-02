use base64;
use chrono::{Local, Utc};
use crypto::{digest::Digest, hmac::Hmac, mac::Mac, md5::Md5, sha1::Sha1};
use failure::{format_err, Error};
use reqwest::header::{HeaderMap, HeaderName, HeaderValue};
use reqwest::{Client as HttpClient, RequestBuilder as ReqwestRequestBuilder};
use std::env;
use std::{borrow::Borrow, str::FromStr};
use url::Url;

/// Default const header.
const DEFAULT_HEADER: &[(&str, &str)] = &[
    ("accept", "application/json"),
    ("x-acs-signature-method", "HMAC-SHA1"),
    ("x-acs-signature-version", "1.0"),
];

#[derive(Debug)]
struct Request {
    action: String,
    uri: String,
    body: Option<String>,
    query: Vec<(String, String)>,
    headers: HeaderMap,
    reqwest_request_builder: ReqwestRequestBuilder,
}

/// The roa style api client.
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
    /// The http client used to send request.
    http: HttpClient,
}

impl Client {
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
            http: HttpClient::new(),
        }
    }

    /// Create a get request builder, and set uri of api.
    pub fn get(&self, uri: &str) -> RequestBuilder {
        let final_url = format!("{}{}", self.endpoint, uri);
        let reqwest_request_builder = self.http.get(&final_url);
        RequestBuilder::new(
            &self.access_key_id,
            &self.access_key_secret,
            &self.endpoint,
            &self.version,
            String::from("GET"),
            String::from(uri),
            reqwest_request_builder,
        )
    }
}

/// The request builder struct.
pub struct RequestBuilder<'a> {
    /// The access key id of aliyun developer account.
    access_key_id: &'a str,
    /// The access key secret of aliyun developer account.
    access_key_secret: &'a str,
    /// The api endpoint of aliyun api service (need start with http:// or https://).
    endpoint: &'a str,
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
        action: String,
        uri: String,
        reqwest_request_builder: ReqwestRequestBuilder,
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
            request: Request {
                action,
                uri,
                body: None,
                query: Vec::new(),
                headers,
                reqwest_request_builder,
            },
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

    /// Set body for request.
    pub fn body(mut self, body: &'static str) -> Self {
        // store body string.
        self.request.body = Some(body.to_string());
        // compute body length and md5.
        let mut md5 = Md5::new();
        let body_bytes = body.as_bytes();
        md5.input(body_bytes);
        let body_md5 = HeaderValue::from_str(&base64::encode(&md5.result_str()));
        let body_length = HeaderValue::from_str(&body_bytes.len().to_string());
        // update headers.
        if let Ok(body_length) = body_length {
            self.request.headers.insert("content-length", body_length);
        }
        if let Ok(body_md5) = body_md5 {
            self.request.headers.insert("content-md5", body_md5);
        }
        self.request.reqwest_request_builder = self.request.reqwest_request_builder.body(body);
        self
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

    /// Send a request to api service.
    pub fn send(mut self) -> Result<String, Error> {
        // gen timestamp.
        let nonce = Local::now().timestamp_subsec_nanos().to_string();
        let ts = Utc::now().format("%a, %d %b %Y %H:%M:%S GMT").to_string();
        self.request.headers.insert("date", ts.parse()?);
        self.request
            .headers
            .insert("x-acs-signature-nonce", nonce.parse()?);

        // parse host of self.endpoint.
        let endpoint = Url::parse(&self.endpoint)?;
        let host = endpoint
            .host_str()
            .ok_or_else(|| format_err!("parse endpoint failed"))?;
        self.request.headers.insert("host", host.parse()?);

        // compute `Authorization` field.
        let authorization = format!("acs {}:{}", self.access_key_id, self.signature());
        self.request
            .headers
            .insert("Authorization", authorization.parse()?);

        // send request.
        let response = self
            .request
            .reqwest_request_builder
            .headers(self.request.headers)
            .query(&self.request.query)
            .send()?
            .text()?;

        // return response.
        Ok(response)
    }

    /// Compute signature for request.
    fn signature(&self) -> String {
        // build body.
        let canonicalized_headers = self.canonicalized_headers();
        let canonicalized_resource = self.canonicalized_resource();
        let body = format!(
            "{}\n{}\n{}\n{}\n{}\n{}\n{}",
            self.request.action.to_uppercase(),
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
        let mut mac = Hmac::new(Sha1::new(), self.access_key_secret.as_bytes());
        mac.input(body.as_bytes());
        let result = mac.result();
        let code = result.code();
        base64::encode(code)
    }

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
}
