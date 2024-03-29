use std::collections::HashMap;
use std::env;
use std::error::Error;

use aliyun_openapi_core_rust_sdk::client::rpc::RPClient;
use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Region {
    region_id: String,
    region_endpoint: String,
    local_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct RegionList {
    request_id: String,
    regions: HashMap<String, Vec<Region>>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // create rpc style api client.
    let aliyun_openapi_client = RPClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        "https://ecs.aliyuncs.com/",
    );

    // call `DescribeRegions` with empty queries, return `RegionList`
    let response = aliyun_openapi_client
        .clone()
        .version("2014-05-26")
        .get("DescribeRegions")
        .json::<RegionList>()
        .await?;
    println!("DescribeRegions response: {response:#?}");

    // call `DescribeInstances` with queries, return `String`
    let response = aliyun_openapi_client
        .version("2014-05-26")
        .get("DescribeInstances")
        .query([("RegionId", "cn-hangzhou")])
        .text()
        .await?;
    println!("DescribeInstances response: {response}");

    Ok(())
}
