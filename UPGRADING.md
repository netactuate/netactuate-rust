# Upgrading to `nars`

There is no previous NetActuate Rust SDK. `nars` is the first Rust client.

## What to add

```toml
[dependencies]
nars = "0.1"
```

Use `Client` for vAPI2 and `V3Client` for vAPI3:

```rust
let v2 = nars::Client::from_env()?;
let servers = v2.list_servers()?;

let v3 = nars::V3Client::from_env()?;
let vpcs = v3.list_vpcs()?;
```

The client reads `NETACTUATE_API_KEY` when a key is not passed explicitly. An empty base URL means
the production NetActuate API.
