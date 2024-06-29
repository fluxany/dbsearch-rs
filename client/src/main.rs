#![allow(unused_assignments)]
#![allow(unused_imports)]
#![allow(dead_code)]

pub mod math;
pub mod search;
pub mod text;
pub mod embed;
pub mod redis_util;
pub mod pdf;
pub mod hashes;

use crate::embed::*;
use crate::pdf::*;
use crate::hashes::*;

use redis::*;
use std::env;
use chatgpt::prelude::*;
use chatgpt::types::*;
use crate::redis_util::*;
use std::io::{Error, Result};
use std::fs::File;
use std::io::{self, Read};
use clap::{App, Arg};
//use num::ToPrimitive;

use serde::{Deserialize, Serialize};
use serde_yaml;
use std::time::Instant;

#[derive(Debug, Deserialize, Serialize)]
pub struct DBSearchConfig {
    pub agent_prompt: String,
    pub query: String,
}

/// Load the configuration file.
pub fn load_config(filename: String) -> std::result::Result<DBSearchConfig, std::io::Error> {
    let yaml_file_r = File::open(filename);
    let mut yaml_file = match yaml_file_r {
        Ok(file) => file,
        Err(e) => {
            println!("Error opening YAML file: {:?}", e);
            return Err(e);
        }
    };

    let mut yaml_content = String::new();
    yaml_file.read_to_string(&mut yaml_content).unwrap();

    //Load the YAML file.
    let config: DBSearchConfig = serde_yaml::from_str(
        &yaml_content.as_str().to_owned()
    ).map_err(|e| io::Error::new(io::ErrorKind::Other, format!("Error parsing YAML: {}", e)))?;
    
    Ok(config)
}

fn concatenate_strings_for_query(strings: Vec<&str>) -> String {
    let mut result = String::new();
    for s in strings {
        result.push_str(format!("{:?}. ", s).as_str());
    }
    result
}

#[tokio::main]
async fn main() -> std::result::Result<(), std::io::Error> {
    // Specify the name of the environment variable you want to retrieve
    let var_name = "OPENAI_API_KEY";
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        return Ok(());
    }

    let matches = App::new("dbsearch")
        .about("DB Search Tool")
        .arg(Arg::with_name("filename")
            .index(1)
            .required(true)
            .help("Sets the input file"))
        .arg(Arg::with_name("config")
            .short('c')
            .required(false)
            .default_value("config.yaml")
            .help("Sets the config file."))
        .arg(Arg::with_name("verbose")
            .short('v')
            .required(false)
            .help("Sets the level of verbosity"))
        .arg(Arg::with_name("host")
            .short('h')
            .required(false)
            .help("Host a manager instance"))
        .get_matches();

    let yaml_filename = matches.value_of("config").unwrap();

    // Get the first argument (index 0 is the program name)
    let file_to_process = matches.value_of("filename").unwrap();
    //let summary_query = &args[2];

    let config = load_config(yaml_filename.to_string()).unwrap();
    let agent_prompt = config.agent_prompt.clone();
    let query = config.query.clone();

    // Get the value of the environment variable
    match env::var(var_name) {
        Ok(val) => {
            println!("The value of {} is: {}", var_name, val);

            // Creating a new ChatGPT client.
            // Note that it requires an API key, and uses
            // tokens from your OpenAI API account balance.
            let client = ChatGPT::new(val).unwrap();

            let mut emb_pairs: Vec<EmbeddingPair> = vec![];
            
            if is_file_processed(file_to_process).await {
                emb_pairs = get_embedding_vectors(file_to_process).await;
                println!("Using {:?} stored embeddings for {:?}", emb_pairs.len(), file_to_process);
            } else {
                emb_pairs = create_embedding_list(file_to_process).await;
                println!("Creating embeddings for: {:?}", file_to_process);
            }

            let start_vecsearch = Instant::now();
            let similar_entries = search_for_similar_entries(
                query.clone(),
                3,
                &mut emb_pairs
            ).await;
            
            let duration_vecsearch = start_vecsearch.elapsed();
            println!("Embedding vector search({:?})", duration_vecsearch);

            let text1 = &similar_entries[0].text;
            let text2 = &similar_entries[1].text;
            let text3 = &similar_entries[2].text;
            concatenate_strings_for_query(vec![text1, text2, text3]);

            //println!("Response: {:?}", response.embeddings());
            let history_array = vec![
                ChatMessage {
                    role: Role::System,
                    content: format!("{}\n\n{:?}", agent_prompt.clone(), text1),
                },
                ChatMessage {
                    role: Role::User,
                    content: format!("{}", &query.clone()),
                },
            ];

            let start = Instant::now();
            let response2 = client
                .send_history(&history_array)
                .await
                .unwrap();
            let duration = start.elapsed();
            
            println!("Response({:?}): {}", duration, response2.message().content);            
        } 
        Err(_) => println!("{} is not defined in the environment.", var_name),
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    /*
    #[test]
    fn test_serdes_1() {
        let data = r#"{\n  \"object\": \"list\",\n  \"data\": [\n    {\n      \"object\": \"embedding\",\n      \"index\": 0,\n      \"embedding\": [\n        -0.0502362,\n        0.036481638,\n    ]\n    }\n  ],\n  \"model\": \"text-embedding-3-small\",\n  \"usage\": {\n    \"prompt_tokens\": 9,\n    \"total_tokens\": 9\n  }\n}\n"#;
        let _v: EmbeddingCompletionResponse = serde_json::from_str(&data).unwrap();
    }
    */

}
