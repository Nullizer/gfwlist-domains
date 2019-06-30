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

    let (lines, errors): (Vec<_>, Vec<_>) = decoded.lines().map(|line| line.trim()).filter(|line| {
        !(
            line.starts_with('!')
            || line.starts_with('@')
            || line.starts_with('[')
            || line.contains(".*")
            || line.is_empty()
        )
    }).map(|line|transform(line, &list)).partition(Result::is_ok);

    let mut lines: Vec<_> = lines.into_iter().map(Result::unwrap).collect();
    lines.sort();
    lines.dedup();

    println!("{}", lines.join("\n"));

    let mut errors: Vec<_> = errors.into_iter().map(Result::unwrap_err).collect();
    errors.sort();

    eprintln!("{}", errors.join("\n"));
    eprintln!("{} lines(deduplicated) transform success!", lines.len());
    eprintln!("{} lines transform failed.", errors.len());
}

fn transform(line: &str, list: &publicsuffix::List) -> Result<String, String> {
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
                Domain(_) => Err(name + " invalid."),
                Ip(_) => Err(name + " is a ip.")
            }
        },
        Err(error) => Err(error.to_string() + ": " + &line)
    }
}
