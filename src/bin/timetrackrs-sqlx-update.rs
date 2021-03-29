use structopt::StructOpt;
use timetrackrs::db::connect_dir;

#[derive(StructOpt)]
struct Args {
    dir: String,
}

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let args = Args::from_args();
    let conn = connect_dir(args.dir, Some(1)).await?;
    conn.acquire().await?;
    println!("ok");
    Ok(())
}
