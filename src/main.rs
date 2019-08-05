#[macro_use]
extern crate clap;
extern crate lib_traxex;

use clap::{App, Arg};
use lib_traxex::download::download;


fn main() {
        let argparse = App::new("traxex")
        .about("traxex, a simple and fast download library similar to wget, written in Rust.")
        .version(crate_version!())
        .author(crate_authors!())
        .arg(Arg::with_name("output")
                .long("output")
                .short("o")
                .takes_value(true)
                .help("Specify the local output filename or directory"))
        .arg(Arg::with_name("url")
            .index(1)
            .required(true))
        .get_matches();
        
    match download(argparse.value_of("url").unwrap(), argparse.value_of("output")) {
        Err(why) => panic!("couldn't write to : {}", why.to_string()),
        Ok(display) => println!("successfully wrote to {}", display)
    }
}
