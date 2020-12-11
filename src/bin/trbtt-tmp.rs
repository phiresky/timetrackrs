#![allow(clippy::all)]

use track_pc_usage_rs::prelude::*;

fn main() -> anyhow::Result<()> {
    let rules = get_default_tag_rule_groups();
    println!("{}", serde_json::to_string(&rules)?);
    Ok(())
}
