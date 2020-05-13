use aliyun_openapi_core_rust_sdk::ROAClient;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::error::Error;
use std::time::Duration;

// roa style client `GET` test.
#[test]
fn roa_client_get_no_query() -> Result<(), Box<dyn Error>> {
    // create roa style api client.
    let aliyun_openapi_client = ROAClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        String::from("https://ros.aliyuncs.com"),
        String::from("2015-09-01"),
    );

    // call `DescribeRegions` with empty queries.
    let response = aliyun_openapi_client.get("/regions").send()?;

    assert!(response.contains("Regions"));

    Ok(())
}

// roa style client `GET` test with timeout.
#[test]
fn roa_client_get_with_timeout() -> Result<(), Box<dyn Error>> {
    // create roa style api client.
    let aliyun_openapi_client = ROAClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        String::from("https://ros.aliyuncs.com"),
        String::from("2015-09-01"),
    );

    // call `DescribeRegions` with empty queries.
    let response = aliyun_openapi_client
        .get("/regions")
        .timeout(Duration::from_millis(1))
        .send();

    assert!(response.is_err());

    Ok(())
}

// roa style client `POST` test.
#[test]
fn roa_client_post_with_json_params() -> Result<(), Box<dyn Error>> {
    // create roa style api client.
    let aliyun_openapi_client = ROAClient::new(
        env::var("ACCESS_KEY_ID")?,
        env::var("ACCESS_KEY_SECRET")?,
        String::from("http://nlp.cn-shanghai.aliyuncs.com"),
        String::from("2018-04-08"),
    );

    // create params.
    let mut params = HashMap::new();
    params.insert("q", "你好");
    params.insert("source", "zh");
    params.insert("target", "en");
    params.insert("format", "text");

    // call `DescribeRegions` with empty queries.
    let response = aliyun_openapi_client
        .post("/nlp/api/translate/standard")
        .header(&[("Content-Type", "application/json")])
        .body(&json!(params).to_string())
        .send()?;

    assert!(response.contains("Hello"));

    Ok(())
}
