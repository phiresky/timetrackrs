use track_pc_usage_rs as trbtt;

fn main() -> anyhow::Result<()> {
    let mut c = trbtt::capture::x11::X11Capturer::init()?;

    let data = c.capture()?;

    /*let z: serde_json::Value = serde_json::from_str(r#"
        {"a": "\u00b7"}
    "#)?;
    println!("z={}", serde_json::to_string_pretty(&z)?);*/

    println!(
        "{}",
        serde_json::to_string_pretty(&data)?
    );


    Ok(())
}
