use clap::Parser;
use lib_traxex::opt::Args;
use lib_traxex::download::download;


fn main() {
    let args = Args::parse();

    match download(&args.url, args.output.as_deref()) {
        Err(why) => panic!("couldn't write to : {}", why),
        Ok(display) => println!("successfully wrote to {}", display)
    }
}
