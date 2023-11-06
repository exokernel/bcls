use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of environment / habitat to use. E.g. int/stg/prd
    #[arg(short, long)]
    env: String,

    /// Long output. Show machine-type, cpu-platform, zone, cell, etc. info.
    /// By default only instance-name and IP are shown.
    /// Can't be used with ip option
    #[arg(short, long, conflicts_with = "ip")]
    long: bool,

    /// Show IP only. Handy for pipeing to other commands like bolt.
    /// Can't be used with long option
    #[arg(short, long, conflicts_with = "long")]
    ip: bool,

    /// Fields

    /// Search pattern to match against instance names. E.g. "^store-lb"
    pattern: String,
}

fn main() {
    let args = Args::parse();

    println!("habitat is {}", args.env);
    println!("-l is {}", args.long);
    println!("-i is {}", args.ip);
    println!("pattern is '{}'", args.pattern);
}
