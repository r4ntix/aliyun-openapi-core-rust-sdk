use aliyun_openapi_core_rust_sdk::ROAClient;
use std::env;
use std::error::Error;

// roa style client test.
#[test]
fn roa_client_no_query() -> Result<(), Box<dyn Error>> {
    // create roa style api client.
    let aliyun_openapi_client = ROAClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        String::from("https://ros.aliyuncs.com"),
        String::from("2015-09-01"),
    );

    // call `DescribeRegions` with empty queries.
    let response = aliyun_openapi_client.get("/regions").send();

    assert!(response.is_ok());

    Ok(())
}
