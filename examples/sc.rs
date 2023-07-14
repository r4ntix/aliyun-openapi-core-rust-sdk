use std::error::Error;
use std::{collections::HashMap, env};

use aliyun_openapi_core_rust_sdk::client::roa::ROAClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct ClusterList {
    clusters: HashMap<String, Vec<Value>>,
    page_index: usize,
    page_size: usize,
    request_id: String,
    total_count: usize,
    total_page: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // create roa style api client.
    let aliyun_openapi_client = ROAClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        "https://foas.cn-hangzhou.aliyuncs.com",
    );

    // call `ListCluster` with empty queries.
    let response = aliyun_openapi_client
        .version("2018-11-11")
        .get("/api/v2/clusters")
        .json::<ClusterList>()
        .await?;

    println!("ListCluster response: {response:#?}");

    Ok(())
}
