use std::env;
use std::fs::File;
use std::io::{self, BufRead, Write};
use futures::stream::*;
use reqwest::Client;
use serde_json::json;

async fn send_to_claude_api(log_content: &str, output_file: &Option<String>) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let api_key = env::var("CLAUDE_API_KEY").expect("CLAUDE_API_KEY not set");
    let url = "https://api.anthropic.com/v1/message";

    let prompt = format!("以下のログを解析し、エラーとワーニングを検出して、それぞれの対策を提案してください。\n\n{}", log_content);

    let response = client
        .post(url)
        .header("Content-Type", "application/json")
        .header("X-API-Key", api_key)
        .header("anthropic-version", "2023-06-01")
        .json(&json!({
            "model": "claude-3-opus-20240229",
            "messages": [
                {
                    "role": "Human",
                    "content": prompt,
                }
            ],
            "max_tokens_to_sample": 1024,
            "stream": true,
        }))
        .send()
        .await?;

    let mut stream = response.bytes_stream();
    let mut file = output_file.as_ref().map(|path| File::create(path).unwrap());

    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        let chunk_str = std::str::from_utf8(&chunk).unwrap();
        
        let lines: Vec<&str> = chunk_str.split('\n').collect();
        for line in lines {
            if line.starts_with("data: ") {
                let data = line[6..].trim();
                if data == "[DONE]" {
                    break;
                }
                let parsed_data: serde_json::Value = serde_json::from_str(data).unwrap();
                if let Some(text) = parsed_data["completion"].as_str() {
                    print!("{}", text);
                    io::stdout().flush().unwrap();

                    if let Some(file) = &mut file {
                        write!(file, "{}", text).unwrap();
                    }
                }
            }
        }
    }

    Ok(())
}

#[tokio::main]
async fn main() {
    let args: Vec<String> = env::args().collect();
    let output_file = args.get(1).cloned();

    let stdin = io::stdin();
    let lines = stdin.lock().lines();
    let mut log_content = String::new();

    for line in lines {
        let line = line.unwrap();
        println!("{}", line);
        log_content.push_str(&line);
        log_content.push('\n');
    }

    if let Err(e) = send_to_claude_api(&log_content, &output_file).await {
        eprintln!("Error: {}", e);
    }
println!("\nComplete");
}
