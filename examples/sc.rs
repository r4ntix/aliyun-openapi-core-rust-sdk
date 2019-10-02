use aliyun_openapi_core_rust_sdk::ROAClient;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // create rpc style api client.
    let aliyun_openapi_client = ROAClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        String::from("https://foas.cn-hangzhou.aliyuncs.com"),
        String::from("2018-11-11"),
    );

    // call `ListCluster` with empty queries.
    let response = aliyun_openapi_client.get("/api/v2/clusters").send()?;
    println!("ListCluster response: {}", response);

    Ok(())
}
