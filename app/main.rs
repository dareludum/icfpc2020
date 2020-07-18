use http_body::Body as _;
use hyper::{Client, Request, Method, Body, StatusCode};
use std::env;
use std::process;

async fn request(server_url: &String, player_key: &String) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    println!("ServerUrl: {}; PlayerKey: {}", server_url, player_key);

    let client = Client::new();
    let req = Request::builder()
        .method(Method::POST)
        .uri(server_url)
        .body(Body::from(format!("{}", player_key)))?;

    match client.request(req).await {
        Ok(mut res) => {
            match res.status() {
                StatusCode::OK => {
                    print!("Server response: ");
                    while let Some(chunk) = res.body_mut().data().await {
                        match chunk {
                            Ok(content) => println!("{:?}", content),
                            Err(why) => println!("error reading body: {:?}", why)
                        }
                    }
                },
                _ => {
                    println!("Unexpected server response:");
                    println!("HTTP code: {}", res.status());
                    print!("Response body: ");
                    while let Some(chunk) = res.body_mut().data().await {
                        match chunk {
                            Ok(content) => println!("{:?}", content),
                            Err(why) => println!("error reading body: {:?}", why)
                        }
                    }
                    process::exit(2);
                }
            }
        },
        Err(err) => {
            println!("Unexpected server response:\n{}", err);
            process::exit(1);
        }
    }

    Ok(())
}


#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let args: Vec<String> = env::args().collect();

    let server_url = &args[1];
    let player_key = &args[2];
    request(server_url, player_key).await
}
