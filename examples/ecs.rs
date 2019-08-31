use aliyun_openapi_core_rust_sdk::RPClient;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // create rpc style api client.
    let aliyun_openapi_client = RPClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        String::from("https://ecs.aliyuncs.com/"),
        String::from("2014-05-26"),
    );

    // call `DescribeRegions` with empty queries.
    let response = aliyun_openapi_client.request(String::from("DescribeRegions"), &[])?;
    println!("DescribeRegions response: {}", response);

    // call `DescribeInstances` with queries.
    let response = aliyun_openapi_client.request(
        String::from("DescribeInstances"),
        &[("RegionId", "cn-hangzhou")],
    )?;
    println!("DescribeInstances response: {}", response);

    Ok(())
}
