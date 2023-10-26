use clap::Parser;

#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Args {
    /// Name of environment / habitat to use. E.g. int/stg/prd
    #[arg(short, long)]
    env: String,

    /// Search pattern for instance names. E.g. "^store-lb"
    pattern: String,
}

fn main() {
    let args = Args::parse();

    println!("habitat is {}", args.env);
    println!("pattern is {}", args.pattern);
}
