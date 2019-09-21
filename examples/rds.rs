use aliyun_openapi_core_rust_sdk::RPClient;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // create rpc style api client.
    let aliyun_openapi_client = RPClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        String::from("https://rds.aliyuncs.com/"),
        String::from("2014-08-15"),
    );

    // call `DescribeRegions` with empty queries.
    let response = aliyun_openapi_client.get("DescribeRegions").send()?;
    println!("DescribeRegions response: {}", response);

    // call `DescribeDBInstances` with queries.
    let response = aliyun_openapi_client
        .get("DescribeDBInstances")
        .query(&[("RegionId", "cn-hangzhou")])
        .send()?;
    println!("DescribeDBInstances response: {}", response);

    Ok(())
}
