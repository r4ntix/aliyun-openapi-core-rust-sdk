use aliyun_openapi_core_rust_sdk::client::rpc::RPClient;
use std::env;
use std::error::Error;

#[tokio::main]
async fn main() -> Result<(), Box<dyn Error>> {
    // create rpc style api client.
    let aliyun_openapi_client = RPClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        "https://vpc.aliyuncs.com/",
    );

    // call `DescribeRegions` with empty queries.
    let response = aliyun_openapi_client
        .clone()
        .version("2016-04-28")
        .get("DescribeRegions")
        .text()
        .await?;
    println!("DescribeRegions response:\n{response}\n");

    // call `DescribeVpcs` with queries.
    let response = aliyun_openapi_client
        .version("2016-04-28")
        .get("DescribeVpcs")
        .query([("RegionId", "cn-hangzhou")])
        .text()
        .await?;
    println!("DescribeVpcs response:\n{response}");

    Ok(())
}
