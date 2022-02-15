use std::{env, fs::File, io::Write, path::Path};

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    let _ = dotenv::dotenv();
    println!("cargo:rerun-if-changed=.env");

    let twitter_id = env::var("TWITTER_CLIENT_ID").expect("Missing env variable TWITTER_CLIENT_ID");
    let twitter_secret =
        env::var("TWITTER_CLIENT_SECRET").expect("Missing env variable TWITTER_CLIENT_SECRET");
    println!("cargo:rerun-if-env-changed=TWITTER_CLIENT_ID");
    println!("cargo:rerun-if-env-changed=TWITTER_CLIENT_SECRET");

    let dest_path = Path::new(&out_dir).join("twitter_credentials.rs");
    let mut file = File::create(dest_path).unwrap();
    writeln!(
        file,
        "const fn twitter_id() -> &'static str {{ {:?} }}",
        twitter_id
    )
    .unwrap();
    writeln!(
        file,
        "const fn twitter_secret() -> &'static str {{ {:?} }}",
        twitter_secret
    )
    .unwrap();
}
