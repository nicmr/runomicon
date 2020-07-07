use std::path::PathBuf;
use std::fs::read_to_string;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {

    let lol_directory: PathBuf = ["F:/", "Games", "Riot Games", "League of Legends"].iter().collect();
    let lockfile_path = lol_directory.join("lockfile");
    
    // read and parse the lockfile
    let lockfile = {
        let contents = read_to_string(lockfile_path)?;
        let v: Vec<&str> = contents.split(':').collect();
        if v.len() < 5 {
            Err("LoL lockfile too short")
        } else {
            Ok(Lockfile {
                process: v[0].to_owned(),
                pid: v[1].parse()?,
                port: v[2].parse()?,
                password: v[3].to_owned(),
                protocol: v[4].to_owned(),
            })
        }
    }?;

    // create the authorization header contents
    let authorization : String = {
        let user_and_password = ["riot", &lockfile.password].join(":");
        let as_b64 = base64::encode(user_and_password);
        ["Basic", &as_b64].join(" ")
    };

    // send the request to the league api
    let resp = reqwest::Client::builder()
        .danger_accept_invalid_certs(true)
        .build()
        .unwrap()
        .get(&(["https://127.0.0.1:", &lockfile.port.to_string(), "/lol-perks/v1/pages"].join("")))
        .header("Authorization",  authorization)
        .send()
        .await?;

    let text =
        resp
        .text()
        .await?;

    println!("{:?}", text);

    Ok(()) 
}

struct Lockfile {
    process: String,
    pid: usize,
    port: usize,
    password: String,
    protocol: String,
}


