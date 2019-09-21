# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added
- ROA style client.

## [0.3.0] - 2019-09-21

### Added
- New function(`get`, `query`, `send`) for RPC style client.

### Deprecated
- `request` function is deprecated, please use the `get` function instead.

## [0.2.0] - 2019-09-01

### Changed
- pass params of `RPClient::request` from by-copy to by-borrow.

## [0.1.0] - 2019-08-31

Initial release.
