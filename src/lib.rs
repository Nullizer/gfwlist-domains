use reqwest;
use base64;
use std::thread;
use std::sync::mpsc;
use publicsuffix::Host::{Domain, Ip};

const PROGRAM_NAME: Option<&str> = option_env!("CARGO_PKG_NAME");

pub struct TransformResult {
    pub domains: Vec<String>,
    pub unhandled: Vec<String>,
    pub dup_count: usize,
}

pub fn direct_get() -> TransformResult {
    let mut can_print = false;
    if let Some("gfwlist-domains") = PROGRAM_NAME {
        can_print = true;
    }

    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let url = "https://raw.githubusercontent.com/gfwlist/gfwlist/master/gfwlist.txt";
        if can_print { eprintln!("Fetching gfwlist from {}", url); }
        let text = reqwest::get(url).unwrap().text().unwrap();
        tx.send(text).unwrap();
    });
    if can_print { eprintln!("Fetching Public Suffix list..."); }
    let list = publicsuffix::List::fetch().unwrap();

    let text = rx.recv().unwrap();
    let decoded = text.lines().map(base64::decode).flat_map(Result::unwrap).collect();
    let decoded = String::from_utf8(decoded).unwrap();

    let (lines, errors): (Vec<_>, Vec<_>) = decoded.lines().map(str::trim).filter(|line| {
        !(
            line.starts_with('!')
            || line.starts_with('@')
            || line.starts_with('[')
            || line.contains(".*")
            || line.is_empty()
        )
    }).map(transform_with(list)).partition(Result::is_ok);

    let mut domains: Vec<_> = lines.into_iter().map(Result::unwrap).collect();
    let before_dedup = domains.len();
    domains.sort();
    domains.dedup();

    let unhandled: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();
    let dup_count = before_dedup - domains.len();

    TransformResult {
        domains,
        unhandled,
        dup_count,
    }
}

fn transform_with(list: publicsuffix::List) -> impl Fn(&str) -> Result<String, String> {
    move |line| {
        let mut line = String::from(line);
        ["||", "|", "http://", "https://", "*", "."].iter().for_each(|search| {
            if line.starts_with(search) {
                line = line.replacen(search, "", 1);
            }
        });
        line = line.replace('*', "/");

        match list.parse_url(&(String::from("http://") + &line)) {
            Ok(host) => {
                let name = host.to_string();
                match host {
                    Domain(ref domain) if domain.has_known_suffix() => Ok(name),
                    Domain(_) => Err(name + " is invalid."),
                    Ip(_) => Err(name + " is a IP.")
                }
            },
            Err(error) => Err(error.to_string() + " in " + &line)
        }
    }
}
