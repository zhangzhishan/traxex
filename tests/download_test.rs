extern crate lib_traxex;

#[cfg(test)]
mod test_download {
    use lib_traxex::download::download;
    use std::path::Path;
    use std::fs;

    #[test]
    fn test_download_no_given_name() {
        let url_str = "https://raw.githubusercontent.com/zhangzhishan/blogpics/dev/traxex.jpg";
        let filename = "traxex.jpg";
        let old_file = Path::new(filename);
        if old_file.exists() {
            fs::remove_file(filename).unwrap();
        }
        match download(url_str, None) {
            Err(why) => panic!("couldn't write to : {}", why),
            Ok(display) => assert_eq!(display, filename)
        }
        assert!(Path::new(filename).exists());
    }

}
