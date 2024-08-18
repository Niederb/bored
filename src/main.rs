use iso8601_duration::Duration;
use rand::thread_rng;
use rand::Rng;
use rusqlite::{params, Connection};
use serde::Deserialize;
use std::fs::File;
use std::io::Read;
use std::path::Path;
use std::time::{SystemTime, UNIX_EPOCH};

fn ensure_database_exists() -> Connection {
    let database_name = "websites.sqlite";
    if !Path::new(database_name).exists() {
        let connection = Connection::open(database_name).unwrap();
        let query = "CREATE TABLE websites (url TEXT NOT NULL UNIQUE, last_access INTEGER);";
        connection.execute_batch(query).unwrap();
        connection
    } else {
        Connection::open(database_name).unwrap()
    }
}

#[derive(Debug, Deserialize)]
struct Website {
    name: String,
    url: String,
    frequency: String,
}

fn main() {
    let mut file = File::open("websites.json").unwrap();
    let mut data = String::new();
    file.read_to_string(&mut data).unwrap();
    let connection = ensure_database_exists();

    let mut websites: Vec<Website> = serde_json::from_str(&data).unwrap();
    println!("{}", websites.len());

    let unix_seconds = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .expect("Time went backwards")
        .as_secs();

    let mut rng = thread_rng();
    loop {
        if websites.is_empty() {
            println!("Nothing more to do. Do something else!");
            break;
        }
        let random_site = rng.gen_range(0..websites.len());

        let w = websites.remove(random_site);
        let duration = w
            .frequency
            .parse::<Duration>()
            .unwrap()
            .num_seconds()
            .unwrap();
        let duration = duration as u64;

        let last_access = get_website_from_db(&connection, &w.url);
        if let Some(last_access) = last_access {
            println!("{} + {} > {}", last_access, duration, unix_seconds);
            if last_access + duration > unix_seconds {
                continue;
            }
        }

        let insert = "INSERT INTO websites(url, last_access) VALUES (?1, ?2) ON CONFLICT(url) DO UPDATE SET last_access=?2;";
        connection
            .execute(insert, params![w.url, unix_seconds])
            .unwrap();
        println!("{} ({}): {}", w.name, w.url, duration);
        let _ = open::that(&w.url);
        break;
    }
}

fn get_website_from_db(c: &Connection, url: &str) -> Option<u64> {
    let query = "SELECT * FROM websites WHERE url = :url";
    let mut stmt = c.prepare(query).ok()?;
    let rows = stmt.query_map(&[(":url", url)], |row| row.get(1)).ok()?;
    match rows.last() {
        Some(value) => value.ok()?,
        None => None,
    }
}
