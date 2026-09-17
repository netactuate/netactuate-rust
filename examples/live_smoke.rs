//! Read only smoke run against a real account. Lists servers and VPCs and prints what came
//! back, so the client is exercised against the platform rather than against a fixture.
//!
//! Run with: NETACTUATE_API_KEY=... cargo run --example live_smoke

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let client = nars::Client::from_env()?;
    let servers = client.list_servers()?;
    println!("vAPI2 list_servers: {} servers", servers.len());
    for s in servers.iter().take(2) {
        println!("    {} {}", s.id, s.name);
    }

    let v3 = nars::V3Client::from_env()?;
    let vpcs = v3.list_vpcs()?;
    println!("vAPI3 list_vpcs:    {} vpcs", vpcs.len());
    for v in vpcs.iter().take(3) {
        println!("    {} {}", v.vpc_id, v.metadata.label);
    }
    Ok(())
}
