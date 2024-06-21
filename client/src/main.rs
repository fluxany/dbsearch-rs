//use std::io;
//use std::path::Path;
pub mod math;
pub mod search;
pub mod text;
pub mod embed;
pub mod redis_util;

//use crate::search::*;
use crate::embed::*;
use redis::*;
use std::env;
use chatgpt::prelude::*;
use chatgpt::types::*;
use crate::redis_util::*;

#[tokio::main]
async fn main() -> Result<()> {
    // Specify the name of the environment variable you want to retrieve
    let var_name = "OPENAI_API_KEY";
    let args: Vec<String> = env::args().collect();

    if args.len() <= 1 {
        return Ok(());
    }
    // Get the first argument (index 0 is the program name)
    let file_to_process = &args[1];
    let summary_query = &args[2];

    // Get the value of the environment variable
    match env::var(var_name) {
        Ok(val) => {
            println!("The value of {} is: {}", var_name, val);

            // Creating a new ChatGPT client.
            // Note that it requires an API key, and uses
            // tokens from your OpenAI API account balance.
            let client = ChatGPT::new(val)?;

            let mut emb_pairs = 
                create_embedding_list(file_to_process)
                .await;

            /*
            let mut con = connect_to_redis().await;
            for i in 0..emb_pairs.len() {
                let key_name = format!("test.bin:{:?}", i);
                let serialized_data : String = 
                    serde_json::to_string(&emb_pairs[i]).unwrap();
                //println!("{:?}", serialized_data);
                con.lpush::<_,_,()>(key_name.clone(), serialized_data)
                    .expect(format!("Unable to process redis command for: {:?}", &key_name.clone()).as_str());
            }
            */
            /*
            let mut i = 0;
            for emb in &emb_pairs {
                let key = format!("{:?}:{:?}", "test.file".to_string(), i);

                i += 1;
            }
            */

            //let query = "Summarize this paper.".to_string();
            let query = summary_query;
            let similar_entries = search_for_similar_entries(
                query.clone(),
                3,
                &mut emb_pairs
            ).await;
            let text1 = &similar_entries[0].text;
            let text2 = &similar_entries[1].text;
            let text3 = &similar_entries[2].text;
        
            //println!("Response: {:?}", response.embeddings());
            let history_array = vec![
                ChatMessage {
                    role: Role::System,
                    content: format!("You are a AI assistant whose expertise is reading and summarizing scientific papers. You are given a query, \
                    a series of text embeddings and the title from a paper in order of their cosine similarity to the query. \
                    You must take the given embeddings and return a very detailed summary of the paper in the languange of the query: \
                    \n\
                    Here are the embeddings: \n\
                    \n\
                    1. {:?}", text1),
                },
                ChatMessage {
                    role: Role::User,
                    content: format!("{}", &query.clone()),
                },
            ];

            let response2 = client
                .send_history(&history_array)
                .await?;           
            
            println!("Response2: {}", response2.message().content);            
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
