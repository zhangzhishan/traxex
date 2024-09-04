use std::path::Path;
use std::path::PathBuf;
use std::fs::File;
use std::fs;
use std::format;
use std::str::FromStr;
use std::convert::TryInto;
use std::cmp::min;


use reqwest::header::CONTENT_DISPOSITION;
use std::collections::HashSet;
use reqwest::header::{HeaderMap, CONTENT_LENGTH, RANGE, HeaderValue};
use reqwest::StatusCode;
use indicatif::{ProgressBar, ProgressStyle};
use anyhow::{bail, Result};


struct PartialRangeIter {
    start: u64,
    end: u64,
    buffer_size: u32,
}

impl PartialRangeIter {
    pub fn new(start: u64, end: u64, buffer_size: u32) -> Result<Self> {
        if buffer_size == 0 {
            bail!("invalid buffer_size, give a value greater than zero.");
        }

        Ok(PartialRangeIter {
            start,
            end,
            buffer_size,
        })
    }
}

impl Iterator for PartialRangeIter {
    type Item = HeaderValue;

    fn next(&mut self) -> Option<Self::Item> {
        if self.start > self.end {
            None
        } else {
            let prev_start = self.start;
            self.start += std::cmp::min(self.buffer_size as u64, self.end - self.start + 1);
            // NOTE(unwrap): `HeaderValue::from_str` will fail only if the value is not made
            // of visible ASCII characters. Since the format string is static and the two
            // values are integers, that can't happen.
            Some(HeaderValue::from_str(&format!("bytes={}-{}", prev_start, self.start - 1)).unwrap())
        }
    }
}

fn filename_fix_existing(filename: &Path) -> String {
    // Expands name portion of filename with numeric ' (x)' suffix to
    // return filename that doesn't exist already.
    let name = filename.file_stem().unwrap().to_str().unwrap();
    tracing::debug!("{}",name);
    let ext = filename.extension().unwrap().to_str().unwrap();
    let dir = filename.parent().unwrap();
    let mut max_index = 0;
    tracing::debug!("{} {:?}", name, dir);
    // if dir is empty, it needs to be current folder, change to "./".
    let dir = if dir.to_str().unwrap().is_empty() { Path::new("./") } else { dir };
    for entry in fs::read_dir(dir).unwrap() {
        let entry = entry.unwrap();
        let path = entry.path();
        if path.is_file() {
            let mut s = String::from(path.file_stem().unwrap().to_str().unwrap());
            if s.starts_with(name) {
                // filter suffixes that match ' (x)' pattern
                let name_start_index = s.find(name).unwrap_or(s.len());
                s.replace_range(name_start_index..name.len(), "");
                s = s.trim().to_string();
                tracing::debug!("name_start_index: {}, s: {}", name_start_index, s);
                tracing::debug!("{}, {}", s.starts_with("("), s.ends_with(")"));
                if s.starts_with("(") && s.ends_with(")") {
                    let index = &s[1..s.len() - 1];
                    if let Ok(int_index) = index.parse::<usize>() {
                        if int_index > max_index {
                            max_index = int_index;
                        }
                    }
                }
            }

        }
    }
    let new_filename = format!("{}_{}.{}", name, max_index + 1, ext);
    new_filename
}


#[test]
fn test_filename_fix_existing() {
    let mut filename = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    filename.push("resources/tests/traxex.jpg");
    assert_eq!(filename_fix_existing(filename.as_path()), "traxex_1.jpg");
}

/// Download a file according to its url.
/// # Examples
/// ```
/// extern crate lib_traxex;
/// use lib_traxex::download::download;
/// fn main() {
///     let url_str = "https://raw.githubusercontent.com/zhangzhishan/blogpics/dev/traxex.jpg";
///     match download(url_str, Some("yourfilename.jpg")) {
///         Err(why) => panic!("couldn't write to : {}", why.to_string()),
///         Ok(display) => println!("successfully wrote to {}", display)
///     }
/// }
/// ```
pub fn download(url: &str, out: Option<&str>) -> Result<String> {
    let client = reqwest::blocking::Client::new();
    let contents = client
        .get(url)
        .send()?;

    let headers = contents.headers().clone();
    tracing::debug!("{}", contents.content_length().unwrap());
    let mut output_dir = "";
    let mut filename = detect_filename(url, &headers);
    if let Some(output) = out {
        if Path::new(output).is_dir() {
            output_dir = output;
        }
        else {
            // if this is not a folder, we will treat it as the filename for output
            filename = output;
        }
    }

    let mut output_filename = PathBuf::from(output_dir);
    output_filename.push(filename);

    let mut path = output_filename.as_path();
    tracing::debug!("path: {}", path.display());
    let new_filename:String;
    if path.exists() {
        new_filename = filename_fix_existing(path);
        output_filename = PathBuf::from(output_dir);
        output_filename.push(new_filename);
        path = output_filename.as_path();
    }
    let display = path.display();
    let mut output_file = match File::create(path) {
        Err(why) => panic!("couldn't create {}: {}", display, why),
        Ok(output_file) => output_file,
    };

    const CHUNK_SIZE: u32 = 10240;

    let response = client.head(url).send()?;
    let length = response
        .headers()
        .get(CONTENT_LENGTH)
        .ok_or_else(|| anyhow::anyhow!("response doesn't include the content length"))?;
    let length = u64::from_str(length.to_str()?).map_err(|_| anyhow::anyhow!("invalid Content-Length header"))?;
    let mut downloaded = 0;
    let pb = ProgressBar::new(length);
    pb.set_style(ProgressStyle::default_bar()
        .template("{spinner:.green} [{elapsed_precise}] [{bar:40.cyan/blue}] {bytes}/{total_bytes} ({eta})")
        .unwrap()
        .progress_chars("#>-"));

    for range in PartialRangeIter::new(0, length - 1, CHUNK_SIZE)? {
        let mut response = client.get(url).header(RANGE, range).send()?;
        let new = min(downloaded + CHUNK_SIZE, length.try_into().unwrap());
        downloaded = new;
        pb.set_position(new.into());

        let status = response.status();
        if !(status == StatusCode::OK || status == StatusCode::PARTIAL_CONTENT) {
            bail!("Unexpected server response: {}", status)
        }

        std::io::copy(&mut response, &mut output_file)?;
    }

    pb.finish_with_message("downloaded");

    Ok(display.to_string())
}

// Return filename for saving file. If no filename is detected from output
// argument, url or headers, return default (download.traxex)
fn detect_filename<'a>(url: &'a str, headers: &'a HeaderMap) -> &'a str {
    let mut filename = "";
    if !headers.is_empty() {
        filename = filename_from_headers(headers);
    }
    if filename.is_empty() && !url.is_empty() {
        filename = filename_from_url(url);
    }
    if !filename.is_empty() {
        filename
    }
    else {
        "download.traxex"
    }
}

// return: detected filename as unicode or None
fn filename_from_url(url: &str) -> &str {
    let url_path = Path::new(url);
    // Get the path filename
    let filename = url_path.file_name().unwrap().to_str().unwrap();
    filename
}

#[test]
fn test_filename_from_url() {
    let filename = filename_from_url("https://raw.githubusercontent.com/zhangzhishan/blogpics/dev/traxex.jpg");
    assert_eq!(filename, "traxex.jpg");
}

// Detect filename from Content-Disposition headers if present.
//     http://greenbytes.de/tech/tc2231/

//     :param: headers as HeaderMap
//     :return: filename from content-disposition header or None
fn filename_from_headers(headers: &HeaderMap) -> &str {
    let mut ret = "";
    if let Some(cdisp) = headers.get(CONTENT_DISPOSITION) {
        let mut cdtype: Vec<&str> = cdisp.to_str().unwrap_or("").split(';').collect();
        let set: HashSet<_> = ["inline".to_string(), "attachment".to_string()].iter().cloned().collect();

        if cdtype.len() > 1 && set.contains(&cdtype[0].trim().to_lowercase()) {
            cdtype.retain(|&val| val.trim().starts_with("filename="));
            if cdtype.len() == 1 {
                // several filename params is illegal, but just in case
                let filenames: Vec<&str> = cdtype[0].split('=').collect();
                let filename = filenames[1].trim();
                if let Some(base_filename) = Path::new(filename).file_name() {
                    ret = base_filename.to_str().unwrap();
                }
            }
        }
    }
    ret
}

#[test]
fn test_filename_from_headers_valid() {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_DISPOSITION, HeaderValue::from_static("attachment; filename=traxex.jpg"));
    let filename = filename_from_headers(&headers);
    assert_eq!(filename, "traxex.jpg");
}

#[test]
fn test_filename_from_headers_invalid() {
    let mut headers = HeaderMap::new();
    headers.insert(CONTENT_DISPOSITION, HeaderValue::from_bytes(&[0x80, 0x80, 0xff, 0xfe, 0xfd]).unwrap());
    let filename = filename_from_headers(&headers);
    assert_eq!(filename, "");
}