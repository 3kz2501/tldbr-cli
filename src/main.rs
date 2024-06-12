use reqwest::header::{HeaderMap, HeaderValue, AUTHORIZATION, CONTENT_TYPE};
use reqwest::Client;
use serde_json::json;
use std::env;
use std::io::{self, Read};

async fn send_to_openai_gpt4(log_content: &str) -> Result<(), reqwest::Error> {
    let client = Client::new();
    let api_key = env::var("OPENAI_API_KEY").expect("OPENAI_API_KEY not set");
    let url = "https://api.openai.com/v1/chat/completions";
    let prompt = format!("以下のログを解析し、エラーとワーニングを検出して、それぞれの対策を提案してください。\n\n{}", log_content);

    let mut headers = HeaderMap::new();
    headers.insert(
        AUTHORIZATION,
        HeaderValue::from_str(&format!("Bearer {}", api_key)).unwrap(),
    );
    headers.insert(CONTENT_TYPE, HeaderValue::from_static("application/json"));

    let response = client
        .post(url)
        .headers(headers)
        .json(&json!({
            "model": "gpt-4o",
            "messages": [
                {
                    "role": "user",
                    "content": prompt,
                }
            ],
            "max_tokens": 1024,
            "stream": false  // ストリーミングを有効にする
        }))
        .send()
        .await?;

    let response_json = response.json::<serde_json::Value>().await?;
    let completion = response_json["choices"][0]["message"]["content"]
        .as_str()
        .unwrap();

    println!("-------------------RESPONSE------------------");
    println!("{}", completion);
    println!("---------------------------------------------");

    Ok(())
}

#[tokio::main]
async fn main() {
    // 標準入力を取得
    let mut stdin = io::stdin();
    // 受け取った全ての内容を格納する文字列変数
    let mut collected_output = String::new();
    stdin.read_to_string(&mut collected_output).unwrap();

    println!("-------------------PIPE INPUT-------------------");
    println!("{}", collected_output);
    println!("------------------------------------------------");

    println!("Start sending to OpenAI GPT-4...");
    let _ = send_to_openai_gpt4(&collected_output).await;

    println!("\nComplete");
}
