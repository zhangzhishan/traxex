use clap::Parser;

#[derive(Parser, Debug, Clone)]
#[clap(author, version, about, long_about = None)]
#[command(about = "traxex, a simple and fast download library similar to wget, written in Rust.")]
pub struct Args {
    #[clap(short, long)]
    pub debug: bool,
    #[clap(short, long)]
    pub output: Option<String>,
    #[clap(index = 1, required = true)]
    pub url: String,
}