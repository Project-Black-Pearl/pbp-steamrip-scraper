#![windows_subsystem = "windows"]
#![feature(option_result_contains)]

use std::{env, path::Path, fs::{self, File}, io::Write};

use scraper::{Html, Selector};

fn main() {
    let query: Vec<String> = env::args().collect();

    if query.len() < 2 {
        println!(r#"You must provide a search query, e.g. "php_steamrip_scraper gta""#);
        std::process::exit(0);
    }

    let query = &query[1];
    let url = format!("https://steamrip.com/?s={}", query);

    let body = reqwest::blocking::get(url).expect("GET Request failed.").text().unwrap();
    let document = Html::parse_document(&body);
    let selector = Selector::parse(r#"div > div > h2 > a"#).unwrap();

    let mut results: Vec<String> = vec![];

    for title in document.select(&selector) {
        if title.html().contains("-free-download/") && results.len() < 6 {
            results.push(format!("https://steamrip.com/{}", title.value().attr("href").unwrap().to_string()));
        }
    }

    for entry in results {
        scan_page(entry);
    }
}

fn scan_page(url: String) {
    let body = reqwest::blocking::get(url).expect("GET Request failed.").text().unwrap();
    let document = Html::parse_document(&body);
    let title_selector = Selector::parse(r#"header > div > h1"#).unwrap();
    let size_selector = Selector::parse(r#"div > ul > li"#).unwrap();
    let download_selector = Selector::parse(r#"p > a"#).unwrap();

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
            downloads.push(download.value().attr("href").unwrap().to_string())
        }
    }

    let title = &titles[0];
    let size = &sizes[0];
    let download = &downloads[0];

    write_to_json(title.to_string(), size.to_string(), download.to_string());
    println!("{}\n{}\nhttps:{}\n", title.to_string(), size.to_string(), download.to_string())
}

fn write_to_json(title: String, size: String, magnet: String) {

    let jsoncontent = format!(
        r#"{{ "title": "{}", "size": {}, "download": "{}" }}
"#,
        title, size, magnet
    );

    let dir_string = format!(r"C:\Users\{}\AppData\Roaming\Project Black Pearl\", whoami::username());
    let dir_path = Path::new(&dir_string);
    let file_string = format!(r"{}\DDL_Cache.json", dir_path.display().to_string());
    let filepath = Path::new(&file_string);

    if !dir_path.exists() {
        fs::create_dir_all(dir_path).unwrap();
    }
    
    if !filepath.exists() {
        File::create(filepath).unwrap();
    }

    let mut file = fs::OpenOptions::new().write(true).append(true).open(filepath).unwrap();
    file.write_all(jsoncontent.as_bytes()).unwrap();
}