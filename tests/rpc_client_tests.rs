use aliyun_openapi_core_rust_sdk::RPClient;
use std::env;
use std::error::Error;
use std::time::Duration;

// rpc style client `GET` test.
#[test]
fn rpc_client_get_no_query() -> Result<(), Box<dyn Error>> {
    // create rpc style api client.
    let aliyun_openapi_client = RPClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        String::from("https://ecs.aliyuncs.com/"),
        String::from("2014-05-26"),
    );

    // call `DescribeRegions` with empty queries.
    let response = aliyun_openapi_client.get("DescribeRegions").send()?;

    assert!(response.contains("Regions"));

    Ok(())
}

// rpc style client `GET` test with query.
#[test]
fn rpc_client_get_with_query() -> Result<(), Box<dyn Error>> {
    // create rpc style api client.
    let aliyun_openapi_client = RPClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        String::from("https://ecs.aliyuncs.com/"),
        String::from("2014-05-26"),
    );

    // call `DescribeInstances` with queries.
    let response = aliyun_openapi_client
        .get("DescribeInstances")
        .query(&[("RegionId", "cn-hangzhou")])
        .send()?;

    assert!(response.contains("Instances"));

    Ok(())
}

// rpc style client `GET` test with timeout.
#[test]
fn rpc_client_get_with_timeout() -> Result<(), Box<dyn Error>> {
    // create rpc style api client.
    let aliyun_openapi_client = RPClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        String::from("https://ecs.aliyuncs.com/"),
        String::from("2014-05-26"),
    );

    // call `DescribeRegions` with empty queries.
    let response = aliyun_openapi_client
        .get("DescribeRegions")
        .timeout(Duration::from_millis(1))
        .send();

    assert!(response.is_err());

    Ok(())
}

// rpc style client `POST` test.
#[test]
fn rpc_client_post_no_query() -> Result<(), Box<dyn Error>> {
    // create rpc style api client.
    let aliyun_openapi_client = RPClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        String::from("https://ecs.aliyuncs.com/"),
        String::from("2014-05-26"),
    );

    // call `DescribeRegions` with empty queries.
    let response = aliyun_openapi_client.post("DescribeRegions").send()?;

    assert!(response.contains("Regions"));

    Ok(())
}

// rpc style client `post` test with query.
#[test]
fn rpc_client_post_with_query() -> Result<(), Box<dyn Error>> {
    // create rpc style api client.
    let aliyun_openapi_client = RPClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        String::from("https://ecs.aliyuncs.com/"),
        String::from("2014-05-26"),
    );

    // call `DescribeInstances` with queries.
    let response = aliyun_openapi_client
        .post("DescribeInstances")
        .query(&[("RegionId", "cn-hangzhou")])
        .send()?;

    assert!(response.contains("Instances"));

    Ok(())
}
