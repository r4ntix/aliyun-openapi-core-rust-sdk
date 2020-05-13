# aliyun-openapi-core-rust-sdk

[![Crates.io](https://img.shields.io/crates/v/aliyun-openapi-core-rust-sdk)](https://crates.io/crates/aliyun-openapi-core-rust-sdk)
[![Documentation](https://docs.rs/aliyun-openapi-core-rust-sdk/badge.svg)](https://docs.rs/aliyun-openapi-core-rust-sdk)
[![Actions Status](https://github.com/r4ntix/aliyun-openapi-core-rust-sdk/workflows/Continuous%20integration/badge.svg)](https://github.com/r4ntix/aliyun-openapi-core-rust-sdk/actions)
[![Crates.io Download Stats](https://img.shields.io/crates/d/aliyun-openapi-core-rust-sdk)](https://crates.io/crates/aliyun-openapi-core-rust-sdk)

Aliyun OpenAPI POP core SDK for Rust.

## Notes

You must know your `AK`(`accessKeyId/accessKeySecret`), and the aliyun product's `endpoint` and `apiVersion`.

For example, The ECS OpenAPI(https://help.aliyun.com/document_detail/25490.html), the API version is `2014-05-26`.

And the endpoint list can be found at [here](https://help.aliyun.com/document_detail/25489.html), the center endpoint is ecs.aliyuncs.com. Add http protocol `http` or `https`, should be `https://ecs.aliyuncs.com/`.

## Usage

The RPC style client:

```rust
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
    let response = aliyun_openapi_client.get("DescribeRegions").send()?;
    println!("DescribeRegions response: {}", response);

    // call `DescribeInstances` with queries.
    let response = aliyun_openapi_client
        .get("DescribeInstances")
        .query(&[("RegionId", "cn-hangzhou")])
        .send()?;
    println!("DescribeInstances response: {}", response);

    Ok(())
}

```

The ROA style client:

```rust
use aliyun_openapi_core_rust_sdk::ROAClient;
use serde_json::json;
use std::collections::HashMap;
use std::env;
use std::error::Error;

fn main() -> Result<(), Box<dyn Error>> {
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

    // call `Translate` with json params.
    let response = aliyun_openapi_client
        .post("/nlp/api/translate/standard")
        .header(&[("Content-Type", "application/json")])
        .body(&json!(params).to_string())
        .send()?;
    println!("Translate response: {}", response);

    Ok(())
}
```

## Examples

Export AK info to env, then run `cargo run --example <NAME>`:

```sh
export ACCESS_KEY_ID=<access_key_id>
export ACCESS_KEY_SECRET=<access_key_secret>

# ecs example
cargo run --example ecs

# rds example
cargo run --example rds

# slb example
cargo run --example slb

# vpc example
cargo run --example vpc
```

## License
The MIT License
