use aliyun_openapi_core_rust_sdk::client::roa::ROAClient;
use std::env;
use std::error::Error;

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
        .text()
        .await?;

    println!("DescribeRegions response:\n{response}");

    Ok(())
}
