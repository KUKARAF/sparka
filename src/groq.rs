use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use crate::calendar::TicketAnalysis;

#[derive(Debug, Serialize)]
struct GroqRequest {
    model: String,
    messages: Vec<GroqMessage>,
    temperature: f32,
    max_tokens: u32,
}

#[derive(Debug, Serialize)]
struct GroqMessage {
    role: String,
    content: String,
}

#[derive(Debug, Deserialize)]
struct GroqResponse {
    choices: Vec<GroqChoice>,
}

#[derive(Debug, Deserialize)]
struct GroqChoice {
    message: GroqMessage,
}

pub async fn analyze_file(content: &str, _access_token: &Option<String>) -> Result<TicketAnalysis> {
    let client = Client::new();
    
    let prompt = format!(
        r#"Analyze this ticket content and extract the following information in JSON format:
{{
    "event_name": "Event name",
    "event_date": "YYYY-MM-DD",
    "event_time": "HH:MM",
    "venue": "Venue name",
    "seat_info": "Seat information if available",
    "ticket_type": "Ticket type if available"
}}

Ticket content:
{}"#,
        content
    );
    
    let request = GroqRequest {
        model: "llama3-70b-8192".to_string(),
        messages: vec![
            GroqMessage {
                role: "system".to_string(),
                content: "You are a ticket analysis assistant. Extract event information from ticket content and return it in the specified JSON format.".to_string(),
            },
            GroqMessage {
                role: "user".to_string(),
                content: prompt,
            },
        ],
        temperature: 0.1,
        max_tokens: 500,
    };
    
    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", "Bearer YOUR_GROQ_API_KEY")
        .header("Content-Type", "application/json")
        .json(&request)
        .send()
        .await?;
    
    let groq_response: GroqResponse = response.json().await?;
    let content = &groq_response.choices[0].message.content;
    
    // Extract JSON from the response
    let json_start = content.find('{').unwrap_or(0);
    let json_end = content.rfind('}').map(|i| i + 1).unwrap_or(content.len());
    let json_str = &content[json_start..json_end];
    
    let analysis: TicketAnalysis = serde_json::from_str(json_str)?;
    Ok(analysis)
}