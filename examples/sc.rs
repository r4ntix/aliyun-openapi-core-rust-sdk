use aliyun_openapi_core_rust_sdk::client::roa::ROAClient;
use std::env;
use std::error::Error;

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
        .text()
        .await?;

    println!("ListCluster response: {}", response);

    Ok(())
}
