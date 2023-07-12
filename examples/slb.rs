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
struct SlbList {
    request_id: String,
    total_count: usize,
    load_balancers: HashMap<String, Vec<Value>>,
    page_size: usize,
    page_number: usize,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // create rpc style api client.
    let aliyun_openapi_client = RPClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        "https://slb.aliyuncs.com/",
    );

    // call `DescribeRegions` with empty queries, return `RegionList`
    let response = aliyun_openapi_client
        .clone()
        .version("2014-05-15")
        .get("DescribeRegions")
        .json::<RegionList>()
        .await?;
    println!("DescribeRegions response: {response:#?}");

    // call `DescribeLoadBalancers` with queries, return `SlbList`
    let response = aliyun_openapi_client
        .version("2014-05-15")
        .get("DescribeLoadBalancers")
        .query([("RegionId", "cn-hangzhou")])
        .json::<SlbList>()
        .await?;
    println!("DescribeLoadBalancers response: {response:#?}");

    Ok(())
}
