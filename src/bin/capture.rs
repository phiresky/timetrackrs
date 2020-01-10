use track_pc_usage_rs as trbtt;

fn main() -> anyhow::Result<()> {
    println!("Hello, world!");
    trbtt::capture::x11::capture()?;


    Ok(())
}
