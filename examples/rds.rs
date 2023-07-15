use std::error::Error;
use std::{collections::HashMap, env};

use aliyun_openapi_core_rust_sdk::client::rpc::RPClient;
use serde::{Deserialize, Serialize};
use serde_json::Value;

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct Region {
    region_id: String,
    region_endpoint: String,
    zone_id: String,
    zone_name: String,
    local_name: String,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct RegionList {
    request_id: String,
    regions: HashMap<String, Vec<Region>>,
}

#[derive(Serialize, Deserialize, Debug)]
#[serde(rename_all = "PascalCase")]
struct DBInstances {
    items: HashMap<String, Vec<Value>>,
    next_token: String,
    page_number: usize,
    page_record_count: usize,
    request_id: String,
    total_record_count: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // create rpc style api client.
    let aliyun_openapi_client = RPClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        "https://rds.aliyuncs.com/",
    );

    // call `DescribeRegions` with empty queries and return `RegionList`
    let response = aliyun_openapi_client
        .clone()
        .version("2014-08-15")
        .get("DescribeRegions")
        .json::<RegionList>()
        .await?;
    println!("DescribeRegions response: {response:#?}");

    // call `DescribeDBInstances` with queries and return `DBInstances`
    let response = aliyun_openapi_client
        .version("2014-08-15")
        .get("DescribeDBInstances")
        .query([("RegionId", "cn-hangzhou")])
        .json::<DBInstances>()
        .await?;
    println!("DescribeDBInstances response: {response:#?}");

    Ok(())
}
