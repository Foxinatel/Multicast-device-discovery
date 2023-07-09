use clap::Parser;

#[derive(clap::Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    #[clap(flatten)]
    mode: Mode,
}

#[derive(clap::Args, Debug)]
#[group(required = true, multiple = false)]
struct Mode {
    /// Operate in client mode
    #[clap(short, long)]
    client: bool,
    /// Operate in server mode
    #[clap(short, long)]
    server: bool,
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let args = Args::parse();
    if args.mode.client {
        client::main()
    } else {
        server::main()
    }
}
