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
//! ```rust
//! use aliyun_openapi_core_rust_sdk::RPClient;
//! use std::error::Error;
//!
//! fn main() -> Result<(), Box<dyn Error>> {
//!     // create rpc style api client.
//!     let aliyun_openapi_client = RPClient::new(
//!         String::from("<access_key_id>"),
//!         String::from("<access_key_secret>"),
//!         String::from("<endpoint>"),
//!         String::from("<version>"),
//!     );
//!
//!     // call `DescribeRegions` with empty queries.
//!     let response = aliyun_openapi_client.get("DescribeRegions").send()?;
//!     println!("DescribeRegions response: {}", response);
//!
//!     // call `DescribeInstances` with queries.
//!     let response = aliyun_openapi_client
//!         .get("DescribeInstances")
//!         .query(&[("RegionId", "cn-hangzhou")])
//!         .send()?;
//!     println!("DescribeInstances response: {}", response);
//!
//!     Ok(())
//! }
//! ```
//!
//! The ROA style client:
//!
//! ```rust
//! unimplemented!();
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

mod rpc;

pub use crate::rpc::Client as RPClient;
