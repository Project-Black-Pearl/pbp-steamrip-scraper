#![feature(option_result_contains)]

use scraper::{Html, Selector};
use std::{
    env,
    fs::{self, File},
    io::Write,
    path::Path,
};

fn main() {

    let args: Vec<String> = env::args().collect();

    if args.len() < 3 {
        println!(r#"You must provide a search query, e.g. "php_steamrip_scraper [query] [destination folder]""#);
        std::process::exit(0);
    }

    let query = &args[1];
    let dest_dir = &args[2];

    let cachestring = dest_dir.to_string();
    let cachedir = Path::new(&cachestring);
    let filename = format!(r"{}\STEAMRIP_Cache.json", cachedir.display());
    let filepath = Path::new(&filename);

    if filepath.exists() {
        println!(r"Clearing previous cache.");
        fs::remove_file(filepath).expect("Couldn't delete config file for re-caching");
    }

    fs::create_dir_all(cachedir).expect("Couldn't create config directory");
    File::create(filepath).expect("Couldn't create config file");

    let mut file = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(filepath)
        .expect("Opening file failed");

    file.write_all(br#"{
    response: [
"#)
        .expect("Couldn't write bytes to file");

    let url = format!("https://steamrip.com/?s={}", query);

    let body = reqwest::blocking::get(url)
        .expect("GET Request failed.")
        .text()
        .expect("Couldn't format body as text.");
    let document = Html::parse_document(&body);
    let selector = Selector::parse(r#"div > div > h2 > a"#).expect("Couldn't parse title.");

    let mut results: Vec<String> = vec![];

    for title in document.select(&selector) {
        if title.html().contains("-free-download/") && results.len() < 6 {
            results.push(format!(
                "https://steamrip.com/{}",
                title
                    .value()
                    .attr("href")
                    .expect("Couldn't fetch game page.")
            ));
        }
    }

    for entry in results {
        scan_page(entry, cachedir);
    }

    file.write_all(br#"    ]
}"#)
        .expect("Couldn't write bytes to file");
}

fn scan_page(url: String, dest_dir: &Path) {
    let body = reqwest::blocking::get(url)
        .expect("GET Request failed.")
        .text()
        .expect("Couldn't format body as text.");
    let document = Html::parse_document(&body);
    let title_selector = Selector::parse(r#"header > div > h1"#).expect("Couldn't parse title.");
    let size_selector = Selector::parse(r#"div > ul > li"#).expect("Couldn't parse filesize.");
    let download_selector = Selector::parse(r#"p > a"#).expect("Couldn't parse download.");

    let mut titles: Vec<String> = vec![];
    let mut sizes: Vec<String> = vec![];
    let mut downloads: Vec<String> = vec![];

    for title in document.select(&title_selector) {
        titles.push(title.inner_html());
    }

    for size in document.select(&size_selector) {
        if size.inner_html().contains("GB")
            || size.inner_html().contains("MB")
            || size.inner_html().contains("KB")
        {
            sizes.push("Size not available for this distributor".to_string());
        }
    }

    for download in document.select(&download_selector) {
        if download.inner_html().contains("DOWNLOAD HERE") {
            downloads.push(
                download
                    .value()
                    .attr("href")
                    .expect("Couldn't append download to vector.")
                    .to_string(),
            )
        }
    }

    let title = &titles[0];
    let size = &sizes[0];
    let download = &downloads[0];

    write_to_json(title.to_string(), size.to_string(), download.to_string(), dest_dir);
}

fn write_to_json(title: String, size: String, magnet: String, dest_dir: &Path) {
    println!("Caching: {}", title);
    let jsoncontent = format!(
        r#"        {{ "title": "{}", "size": "{}", "download": "https:{}" }},
"#,
        title, size, magnet
    );

    let dir_string = dest_dir;
    let dir_path = Path::new(&dir_string);
    let file_string = format!(r"{}\STEAMRIP_Cache.json", dir_path.display());
    let file_path = Path::new(&file_string);

    let mut file = fs::OpenOptions::new()
        .write(true)
        .append(true)
        .open(file_path)
        .expect("Opening file failed");

    file.write_all(jsoncontent.as_bytes())
        .expect("Couldn't write bytes to file");
}
