mod wildcard;
use wildcard::Wildcard;

fn main() {
    let wildcard = Wildcard::parse("blarg[!!xy0-9a-z.[]/*.JP?").unwrap();
    println!("{}", wildcard);
    println!("{:?}", wildcard);
}
