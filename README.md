# nars

`nars` is the Rust SDK for NetActuate vAPI2 and vAPI3.

It currently covers the SDK foundation and these endpoint families:

- vAPI2: cloud servers, DNS zones and DNS records
- vAPI3: VPCs, storage buckets and NKE clusters

## Install

```toml
[dependencies]
nars = "0.1"
```

## Authenticate

Pass an API key explicitly or let the client read `NETACTUATE_API_KEY`.

```rust
let v2 = nars::Client::from_env()?;
let v3 = nars::V3Client::from_env()?;
```

An empty base URL selects production. Use `with_base_url` to point at a test or staging API.

```rust
let v3 = nars::V3Client::with_base_url("api-key", "https://vapi3.example.test")?;
```

API keys are redacted from error messages and request display paths.

## Two Clients

vAPI2 and vAPI3 use different response envelopes, pagination and error shapes, so the SDK exposes two clients:

- `Client` for vAPI2, production base `https://vapi2.netactuate.com/api/`
- `V3Client` for vAPI3, production base `https://vapi3.netactuate.com`

The method names follow the shared SDK contract in Rust style, for example `get_server`,
`list_vpcs` and `get_nke_cluster`.

## Features

The default `blocking` feature enables the blocking reqwest transport. The `async` feature exposes
async reqwest construction hooks for applications that use an async runtime.

See https://netactuate.com/docs for platform documentation.
