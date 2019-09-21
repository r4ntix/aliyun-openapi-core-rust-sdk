use aliyun_openapi_core_rust_sdk::RPClient;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // create rpc style api client.
    let aliyun_openapi_client = RPClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        String::from("https://vpc.aliyuncs.com/"),
        String::from("2016-04-28"),
    );

    // call `DescribeRegions` with empty queries.
    let response = aliyun_openapi_client.get("DescribeRegions").send()?;
    println!("DescribeRegions response: {}", response);

    // call `DescribeVpcs` with queries.
    let response = aliyun_openapi_client
        .get("DescribeVpcs")
        .query(&[("RegionId", "cn-hangzhou")])
        .send()?;
    println!("DescribeVpcs response: {}", response);

    Ok(())
}
