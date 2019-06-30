use reqwest;
use base64;
use std::thread;
use std::sync::mpsc;
use publicsuffix::Host::{Domain, Ip};

fn main() {
    let (tx, rx) = mpsc::channel();
    thread::spawn(move || {
        let url = "https://raw.githubusercontent.com/gfwlist/gfwlist/master/gfwlist.txt";
        let text = reqwest::get(url).unwrap().text().unwrap();
        tx.send(text).unwrap();
    });
    let list = publicsuffix::List::fetch().unwrap();

    let text = rx.recv().unwrap();
    let decoded = text.lines().flat_map(|line| base64::decode(line).unwrap()).collect();
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

    let mut lines: Vec<_> = lines.into_iter().map(Result::unwrap).collect();
    let before_dedup = lines.len();
    lines.sort();
    lines.dedup();
    println!("{}", lines.join("\n"));

    let errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();
    eprintln!("\n{}\n", errors.join("\n"));
    eprintln!("Total {} lines (already removed {} duplicated) transformed.",
        lines.len(), before_dedup - lines.len());
    eprintln!("{} lines can't transform.", errors.len());
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
