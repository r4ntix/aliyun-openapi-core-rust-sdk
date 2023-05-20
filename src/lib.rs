//! Aliyun OpenAPI POP core SDK for Rust.
//!
//! # Notes
//!
//! You must know your `AK`(`accessKeyId/accessKeySecret`), and the aliyun product's `endpoint` and `apiVersion`.
//!
//! For example, The ECS OpenAPI(https://help.aliyun.com/document_detail/25490.html), the API version is `2014-05-26`.
//!
//! And the endpoint list can be found at [here](https://help.aliyun.com/document_detail/25489.html), the center endpoint is ecs.aliyuncs.com. Add http protocol `http` or `https`, should be `http://ecs.aliyuncs.com/`.
//!
//! # Usage
//!
//! The RPC style client:
//!
//! ```no_run
//! use aliyun_openapi_core_rust_sdk::client::rpc::RPClient;
//! use std::error::Error;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     // create rpc style api client.
//!     let aliyun_openapi_client = RPClient::new(
//!         "<access_key_id>",
//!         "<access_key_secret>",
//!         "<endpoint>",
//!     );
//!
//!     // call `DescribeInstances` with queries.
//!     let response = aliyun_openapi_client
//!         .get("DescribeInstances")
//!         .query([("RegionId", "cn-hangzhou")])
//!         .text().await?;
//!
//!     println!("DescribeInstances response: {response}");
//!
//!     Ok(())
//! }
//! ```
//!
//! The ROA style client:
//!
//! ```no_run
//! use aliyun_openapi_core_rust_sdk::client::roa::ROAClient;
//! use serde_json::json;
//! use std::collections::HashMap;
//! use std::env;
//! use std::error::Error;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn Error>> {
//!     // create roa style api client.
//!     let aliyun_openapi_client = ROAClient::new(
//!         env::var("ACCESS_KEY_ID")?,
//!         env::var("ACCESS_KEY_SECRET")?,
//!         "http://mt.aliyuncs.com",
//!     );
//!
//!     // create params.
//!     let mut params = HashMap::new();
//!     params.insert("SourceText", "你好");
//!     params.insert("SourceLanguage", "zh");
//!     params.insert("TargetLanguage", "en");
//!     params.insert("FormatType", "text");
//!     params.insert("Scene", "general");
//!
//!     // call `Translate` with json params.
//!     let response = aliyun_openapi_client
//!         .version("2018-04-08")
//!         .post("/api/translate/web/general")
//!         .header([("Content-Type".to_string(), "application/json".to_string())])?
//!         .body(json!(params).to_string())?
//!         .text()
//!         .await?;
//!
//!     println!("Translate response: {response}");
//!
//!     Ok(())
//! }
//! ```
//! # Examples
//!
//! Export AK info to env, then run `cargo run --example <NAME>`:
//!
//! ```sh
//! export ACCESS_KEY_ID=<access_key_id>
//! export ACCESS_KEY_SECRET=<access_key_secret>
//!
//! # ecs example
//! cargo run --example ecs
//!
//! # rds example
//! cargo run --example rds
//!
//! # slb example
//! cargo run --example slb
//!
//! # vpc example
//! cargo run --example vpc
//! ```

mod roa;
mod rpc;

#[deprecated(
    since = "1.0.0",
    note = "Please use the `aliyun_openapi_core_rust_sdk::client::roa::ROAClient` function instead"
)]
pub use crate::roa::Client as ROAClient;

#[deprecated(
    since = "1.0.0",
    note = "Please use the `aliyun_openapi_core_rust_sdk::client::rpc::RPClient` function instead"
)]
pub use crate::rpc::Client as RPClient;

pub mod client;

pub trait OpenAPI<'a> {
    type Output: 'a;

    /// Create a `GET` request with the `uri`.
    ///
    /// Returns a `RequestBuilder` for send request.
    fn get(&'a self, uri: &str) -> Self::Output {
        self.execute("GET", uri)
    }

    /// Create a `POST` request with the `uri`.
    ///
    /// Returns a `RequestBuilder` for send request.
    fn post(&'a self, uri: &str) -> Self::Output {
        self.execute("POST", uri)
    }

    /// Create a `PUT` request with the `uri`.
    ///
    /// Returns a `RequestBuilder` for send request.
    fn put(&'a self, uri: &str) -> Self::Output {
        self.execute("PUT", uri)
    }

    /// Create a request with the `method` and `uri`.
    ///
    /// Returns a `RequestBuilder` for send request.
    fn execute(&'a self, method: &str, uri: &str) -> Self::Output;
}
