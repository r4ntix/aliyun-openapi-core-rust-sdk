use aliyun_openapi_core_rust_sdk::RPClient;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
    // create rpc style api client.
    let aliyun_openapi_client = RPClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        String::from("https://slb.aliyuncs.com/"),
        String::from("2014-05-15"),
    );

    // call `DescribeRegions` with empty queries.
    let response = aliyun_openapi_client.get("DescribeRegions").send()?;
    println!("DescribeRegions response: {}", response);

    // call `DescribeLoadBalancers` with queries.
    let response = aliyun_openapi_client
        .get("DescribeLoadBalancers")
        .query(&[("RegionId", "cn-hangzhou")])
        .send()?;
    println!("DescribeLoadBalancers response: {}", response);

    Ok(())
}
