# Traxex
[![LICENSE](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
[![Crates.io Version](https://img.shields.io/crates/v/traxex.svg)](https://crates.io/crates/traxex)
[![Crates.io](https://img.shields.io/crates/d/traxex.svg)](https://crates.io/crates/traxex)
[![Coverage Status](https://coveralls.io/repos/github/zhangzhishan/traxex/badge.svg?branch=master)](https://coveralls.io/github/zhangzhishan/traxex?branch=master)

Linux: [![Build Status](https://travis-ci.org/zhangzhishan/traxex.svg?branch=master)](https://travis-ci.org/zhangzhishan/traxex)
Windows: [![Build status](https://ci.appveyor.com/api/projects/status/8vje826k0p1e415l/branch/master?svg=true)](https://ci.appveyor.com/project/zhangzhishan/traxex/branch/master)
## Introduction
Traxex is the name of a Dota hero my wife like most, so I chose this as the name. When I try to download some files in another rust application, which I am working on. I couldn't find a easy to use library for downloading like wget or something similar. Therefore, I wrote this library, a very easy download library. As I am a newbee in rust, so there may be some code can be improved. Welcome to give me some advice by issues or pull request. Thank you in advance.
## Usage
There are a binary which can be used to download files through url and a library to be used in your code.
### Binary Usage

```
USAGE:
    traxex.exe [OPTIONS] <url>

FLAGS:
    -h, --help       Prints help information
    -V, --version    Prints version information

OPTIONS:
    -o, --output <output>    Specify the local output filename or directory

ARGS:
    <url>
```
### Library Usage
There is only one public method to use `download`. This method has two params, the first is a `&str` url link, and the second is the output folder or output filename `Option<&str>`. If you don't want to give the filename, you can just leave the second parameter as `None`. It will generate a given filename according to the url path or from Content-Disposition headers if present. This method can return a `Result<String>`, which is the filename of the downloaded file.

```
extern crate lib_traxex;
use lib_traxex::download::download;

fn main() {
    let url_str = "https://raw.githubusercontent.com/zhangzhishan/blogpics/dev/traxex.jpg";

    match download(url_str, None) {
        Err(why) => panic!("couldn't write to : {}", why.to_string()),
        Ok(display) => println!("successfully wrote to {}", display)
    }
}
```
## Reference
[Python wget](https://bitbucket.org/techtonik/python-wget/src/default/)
