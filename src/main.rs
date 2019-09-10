mod auth;

fn main() {
    let token = auth::authorize().unwrap();
    println!("{}", token);
}
