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
    let model = Llama::new_chat();
    let seed = read_prompt_from_file("seed.txt");
    let mut last_generated = String::from("ROBOT:\n...\n");
    loop {
        println!("\n\nEnter a prompt: ");
        let user_prompt = format!("{}\n{}\nUSER:\n{}\nROBOT:\n", seed, last_generated, read_user_input());
        // println!("\n\n<DEBUG(prompt)>\n{}</DEBUG>\n\n", user_prompt);
        let mut result = model.stream_text(&user_prompt).await.expect("Failed to stream text");
        println!();
        last_generated = String::from("ROBOT:\n");
        while let Some(token) = result.next().await {
            print!("{token}");
            last_generated.push_str(&format!("{token}"));
        }
        last_generated.push_str("\n");
    }
}
