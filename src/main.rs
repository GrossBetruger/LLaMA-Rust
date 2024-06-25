use std::io::Read;
use kalosm_llama::prelude::*;
use tokio;


fn read_user_input() -> String {
    let mut input = String::new();
    std::io::stdin().read_line(&mut input).expect("Failed to read line");
    input
}


fn read_prompt_from_file(path: &str) -> String {
    let mut file = std::fs::File::open(path).expect(format!("{}{}", "Failed to open file: ", path).as_str());
    let mut contents = String::new();
    file.read_to_string(&mut contents).expect("failed to read file");
    contents
}


#[tokio::main]
async fn main() {
    let model = Llama::default();
    let seed = read_prompt_from_file("seed.txt");
    println!("Enter a prompt: ");
    let prompt = read_user_input();

    let mut result = model.stream_text(&format!("{}\n\n{}", seed, prompt)).await.expect("Failed to stream text");
    println!();

    while let Some(token) = result.next().await {
        print!("{token}");
    }
}
