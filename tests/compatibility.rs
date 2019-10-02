#![allow(deprecated)]

use aliyun_openapi_core_rust_sdk::RPClient;
use std::env;
use std::error::Error;

// 0.2.0 version, rpc style client test.
#[test]
fn rpc_client_no_query_compatibility_020() -> Result<(), Box<dyn Error>> {
    // create rpc style api client.
    let aliyun_openapi_client = RPClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        String::from("https://ecs.aliyuncs.com/"),
        String::from("2014-05-26"),
    );

    // call `DescribeRegions` with empty queries.
    let response = aliyun_openapi_client.request("DescribeRegions", &[]);

    assert!(response.is_ok());

    Ok(())
}

// 0.2.0 version, rpc style client test with query.
#[test]
fn rpc_client_with_query_compatibility_020() -> Result<(), Box<dyn Error>> {
    // create rpc style api client.
    let aliyun_openapi_client = RPClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        String::from("https://ecs.aliyuncs.com/"),
        String::from("2014-05-26"),
    );

    // call `DescribeInstances` with queries.
    let response =
        aliyun_openapi_client.request("DescribeInstances", &[("RegionId", "cn-hangzhou")]);

    assert!(response.is_ok());

    Ok(())
}
