use aliyun_openapi_core_rust_sdk::client::log_service::LogServiceClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;
use std::env;
use std::error::Error;

#[derive(Serialize, Deserialize, Debug)]
struct ProjectList {
    projects: Value,
    count: usize,
    total: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // create log service api client.
    let aliyun_openapi_client = LogServiceClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        "https://cn-hangzhou.log.aliyuncs.com",
    );

    // call `ListProject` api.
    let response = aliyun_openapi_client
        .clone()
        .get("/")
        .json::<ProjectList>()
        .await?;
    println!("ListProject response: {response:#?}");

    // call `GetProject` api and parse error
    let err = aliyun_openapi_client
        .get("/")
        .project("project_name")
        .text()
        .await
        .unwrap_err();
    println!("GetProject response err: {err:#?}");

    Ok(())
}
