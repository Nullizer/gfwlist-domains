fn main() {
    let result = gfwlist_domains::direct_get();
    println!("{}", result.domains.join("\n"));

    eprintln!("\nNot transform:\n\n{}\n", result.unhandled.join("\n"));
    eprintln!("Total {} lines (already removed {} duplicated) transformed.",
        result.domains.len(), result.dup_count);
    eprintln!("{} lines can't transform.", result.unhandled.len());
}
