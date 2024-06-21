use crate::text::*;
use crate::math::*;
use std::sync::{Arc, Mutex};
use std::env;
use chatgpt::prelude::*;
use chatgpt::types::*;
use rayon::prelude::*;
use tokio::runtime::Runtime;
use serde_json::*;
use std::cmp::Ordering;
use serde::{Serialize, Deserialize};
//use tokio::time::{delay_for, Duration};
use std::thread;
use std::time::Duration;
use redis::*;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EmbeddingPair {
    pub text: String,
    pub embedding: Vec<f32>,
    pub similarity: f32
}

impl EmbeddingPair {
    pub fn new(text: String, embedding: Vec<f32>) -> EmbeddingPair {
        EmbeddingPair {
            text: text,
            embedding: embedding,
            similarity: 0.0,
        }
    }
}

pub async fn gpt_get_embeddings(text: &String) -> std::result::Result<Vec<f32>, chatgpt::err::Error> {
    // Specify the name of the environment variable you want to retrieve
    let var_name = "OPENAI_API_KEY";

    // Get the value of the environment variable
    match env::var(var_name) {
        Ok(val) => {
            // Creating a new ChatGPT client.
            // Note that it requires an API key, and uses
            // tokens from your OpenAI API account balance.
            let client = ChatGPT::new(val).unwrap();
            let response: EmbeddingCompletionResponse = client
                .get_embeddings(&text)
                .await?;
            println!("Getting embeddings");
            Ok(response.embeddings().clone())
        } 
        Err(_) => {
            println!("{} is not defined in the environment.", var_name);            
            Ok(vec![])
        }
    }
}

pub async fn create_embedding_list (filename: &str) -> Vec<EmbeddingPair> {
    let pdf_text = extract_pdf_text(filename);
    let mut text_summary: TextSummary = TextSummary::new(pdf_text);
    let text_list = text_summary.tokenize_words_into_chunks(
        400, 100
    );
    println!("Getting total of {} text pairs", text_list.len());
    let pair_list: Arc<Mutex<Vec<EmbeddingPair>>> = Arc::new(Mutex::new(Vec::new()));
    let redis_connection: redis::Connection = crate::redis_util::connect_to_redis().await;
    let redis_arc: Arc<Mutex<redis::Connection>> = Arc::from(Mutex::from(redis_connection));            
    let max_workers = 16; // Define the maximum number of workers (threads) to use
    let redis_count: Arc<Mutex<u32>> = Arc::from(Mutex::from(0 as u32));
    let pool = rayon::ThreadPoolBuilder::new()
        .num_threads(max_workers)
        .build()
        .unwrap();
        
        pool.install(|| {
            let rt = Runtime::new().unwrap();
            text_list.par_iter().for_each(|text_list_item| {
                thread::sleep(Duration::from_millis(500));
                if let Ok(embedding) = rt.block_on(gpt_get_embeddings(text_list_item)) {
                    let new_pair = EmbeddingPair {
                        text: text_list_item.clone(),
                        embedding,
                        similarity: 0.0,
                    };
                    pair_list.lock().unwrap().push(new_pair.clone());
                    let serialized_data : String = serde_json::to_string(&new_pair).unwrap();
                    let mut c = redis_count.lock().unwrap();
                    *c += 1;
                    
                    let key_name = format!("test.bin:{:?}", c);
                    redis_arc.lock().unwrap().lpush::<_,_,()>(key_name.clone(), serialized_data);
                }
            });
        });

    Arc::try_unwrap(pair_list).unwrap().into_inner().unwrap()
}

pub async fn search_for_similar_entries(
    query: String,
    num_similar_entries: usize,
    pairs: &mut Vec<EmbeddingPair>
) -> Vec<EmbeddingPair> {
    let mut result : Vec<EmbeddingPair> = Vec::new();
    if let Ok(emb) = gpt_get_embeddings(&query).await {
        for pair in pairs.iter_mut() {
            pair.similarity = 
                cosine_similarity(&emb, &pair.embedding);            
        }

        // Sort pairs by similarity (higher similarity first)
        pairs.sort_by(|a, b| b.similarity.partial_cmp(&a.similarity).unwrap_or(Ordering::Equal));

        // Take the top num_similar_entries pairs
        result.extend_from_slice(&pairs[..num_similar_entries]);
    }
    result
}