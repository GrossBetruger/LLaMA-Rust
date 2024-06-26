use std::io::Read;

use clap::Parser;
use kalosm_llama::prelude::*;
use regex::Regex;
use tokio;

const _END_OF_TURN_TOKEN: &str = &"ROBOT:";
const _TEST_PROMPT: &str = &"A vulnerability was found in Schneider Electric APC Easy UPS Online up to 2 --- A vulnerability, which was classified as critical, was found in Apple Safari up to 15 --- A vulnerability classified as critical was found in D-Link DIR-895 FW102b07 (Router Operating System) --- A vulnerability, which was classified as critical, was found in Microsoft Edge 99 --- A vulnerability classified as problematic was found in Huawei HarmonyOS and EMUI (affected version not known)";
const USER_IMPERSONATE_PATTERN: &str = r"(?s)USER:\n.+";

#[derive(clap::ValueEnum, Clone, Default, Debug)]
enum ChatMode {
    #[default]
    General, // General chat bot mode
    TaskSpecific, // Task specific QUESTION: / ANSWER: chat mode (requires prompt with examples)
}

#[derive(Parser, Debug)]
#[command(version, about, long_about = None)]
struct Args {
    /// chat bot mode
    #[arg(short, long)]
    mode: ChatMode,
}

fn read_user_input() -> String {
    let mut input = String::new();
    std::io::stdin()
        .read_line(&mut input)
        .expect("Failed to read line");
    input
}

fn read_prompt_from_file(path: &str) -> String {
    let mut file =
        std::fs::File::open(path).expect(format!("{}{}", "Failed to open file: ", path).as_str());
    let mut contents = String::new();
    file.read_to_string(&mut contents)
        .expect("failed to read file");
    contents
}

fn clean_garbage_text(text: &str, garbage_patterns: Vec<&str>) -> String {
    let mut cleaned_text = String::from(text);
    for pattern in garbage_patterns {
        let re = Regex::new(pattern).expect(&format!("Failed to create regex pattern {}", pattern));
        cleaned_text = re.replace_all(&cleaned_text, "").to_string();
    }
    cleaned_text
}

fn squash_linebreaks(text: &str) -> String {
    let re = Regex::new(r"\n{2,}").expect("Failed to create regex pattern");
    re.replace_all(text, "\n").to_string()
}

const ROBOT_TURN_HEADER: &'static str = "ROBOT:\n";

const USER_TURN_HEADER: &'static str = "USER:\n";

async fn chat_main(model: Llama, seed: String) {
    let mut last_generated = String::from("ROBOT:\n...\n");
    loop {
        println!("\n\nEnter a prompt: ");
        let user_prompt = format!(
            "{}\n{}\n{}{}\nROBOT:\n",
            seed,
            last_generated,
            USER_TURN_HEADER,
            read_user_input()
        );
        // println!("\n\n<DEBUG(prompt)>\n{}</DEBUG>\n\n", user_prompt);
        let mut result = model
            .stream_text(&user_prompt)
            .await
            .expect("Failed to stream text");
        println!();
        last_generated = String::from(ROBOT_TURN_HEADER);
        while let Some(token) = result.next().await {
            // print!("{token}");
            last_generated.push_str(&format!("{token}"));
        }
        last_generated = clean_garbage_text(&last_generated, vec![USER_IMPERSONATE_PATTERN]);
        last_generated.push_str("\n");
        last_generated = squash_linebreaks(&last_generated);
        println!("{}", last_generated);
    }
}

async fn task_specific_chat(model: Llama, seed: String) {
    let mut last_generated_history = Vec::new();
    loop {
        println!("\n\nEnter a prompt: ");
        let user_input = read_user_input();
        let prompt = format!("{}\nQUESTION:\n\n{}\nANSWER:\n\n", seed, user_input);
        // println!("\n\n<DEBUG(prompt)>\n{}</DEBUG>\n\n", prompt);
        let mut result = model
            .stream_text(&prompt)
            .await
            .expect("Failed to stream text");
        println!();
        let mut last_generated = String::from("ROBOT:\n");
        while let Some(token) = result.next().await {
            print!("{token}");
            last_generated.push_str(&format!("{token}"));
        }
        last_generated.push_str("\n");
        last_generated_history.push(last_generated);
    }
}

#[tokio::main]
async fn main() {
    let args = Args::parse();

    match args.mode {
        ChatMode::General => {
            let model = Llama::new_chat();
            let seed = read_prompt_from_file("seed.txt");
            chat_main(model, seed).await;
        }
        ChatMode::TaskSpecific => {
            let model = Llama::new_chat();
            let seed = read_prompt_from_file("prompt_learn_from_example.txt");
            task_specific_chat(model, seed).await;
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_clean_garbage_text() {
        let text = "USER:\n hello kind robot? \nROBOT:\n hello kind human!";
        let cleaned_text = clean_garbage_text(text, vec![USER_IMPERSONATE_PATTERN]);
        assert_eq!(cleaned_text, "");
    }

    #[test]
    fn test_squash_linebreaks() {
        let text = "ROBOT:\nwould you like an omelette?\n\n\n";
        let squashed_text = squash_linebreaks(text);
        assert_eq!(squashed_text, "ROBOT:\nwould you like an omelette?\n");
    }

    #[test]
    fn test_read_prompt_from_file() {
        let path = "test_seed.txt";
        let prompt = read_prompt_from_file(path);
        assert_eq!(prompt, "test seed\n");
    }
}
