use regex::Regex;
fn main() {
    let pat = "(?<name>a)|(?<name>b)";
    let re = Regex::new(r"\(\?<[^>]+>").unwrap();
    let clean = re.replace_all(pat, "(");
    println!("{}", clean);
}
