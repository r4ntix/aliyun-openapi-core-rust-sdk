use aliyun_openapi_core_rust_sdk::client::roa::ROAClient;
use serde::{Deserialize, Serialize};
use std::env;
use std::error::Error;

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
    regions: Vec<Region>,
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // create roa style api client.
    let aliyun_openapi_client = ROAClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        "https://ros.aliyuncs.com",
    );

    // call `DescribeRegions` with empty queries.
    let response = aliyun_openapi_client
        .version("2015-09-01")
        .get("/regions")
        .json::<RegionList>()
        .await?;

    println!("DescribeRegions response: {response:#?}");

    Ok(())
}
