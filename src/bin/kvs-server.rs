use clap::Parser;
use kvs::engine::Result;

#[derive(Parser)]
#[command(author, version, about, long_about = None)]
struct Args {
    #[arg(short, long, value_name = "IP-PORT", default_value_t = String::from("127.0.0.1:4000"))]
    addr: String,
    #[arg(short, long, default_value_t = String::from("kvs"))]
    engine: String
}

fn main() -> Result<()> {
    let args = Args::parse();
    println!("The addr is {}.", args.addr);
    println!("The engine is {}.", args.engine);
    Ok(())
}