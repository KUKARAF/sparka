use anyhow::Result;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, Duration, Datelike, Weekday};
use crate::calendar::CalendarEvent;

#[derive(Debug, Serialize, Deserialize)]
pub struct SchedulingGoal {
    pub id: String,
    pub user_id: String,
    pub goal_type: GoalType,
    pub description: String,
    pub frequency: Frequency,
    pub duration_minutes: u32,
    pub preferred_times: Vec<TimePreference>,
    pub created_at: DateTime<Utc>,
    pub is_active: bool,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum GoalType {
    Exercise,
    Hobby,
    Learning,
    Social,
    Work,
    Custom(String),
}

#[derive(Debug, Serialize, Deserialize)]
pub enum Frequency {
    Daily,
    Weekly,
    Monthly,
    Custom(u32), // Number of times per period
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TimePreference {
    pub day_of_week: Option<Weekday>,
    pub start_time: String, // HH:MM format
    pub end_time: String,   // HH:MM format
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScheduleSuggestion {
    pub id: String,
    pub goal_id: String,
    pub title: String,
    pub description: String,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub confidence_score: f32, // 0.0 to 1.0
    pub reasoning: String,
    pub created_at: DateTime<Utc>,
    pub status: SuggestionStatus,
}

#[derive(Debug, Serialize, Deserialize)]
pub enum SuggestionStatus {
    Pending,
    Accepted,
    Rejected,
    Expired,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ScheduleRequest {
    pub goal_description: String,
    pub existing_events: Vec<CalendarEvent>,
    pub preferences: Vec<TimePreference>,
    pub duration_minutes: u32,
}

pub struct SuggestionEngine;

impl SuggestionEngine {
    pub fn new() -> Self {
        Self
    }
    
    pub async fn generate_suggestions(
        &self,
        request: &ScheduleRequest,
        access_token: &str,
    ) -> Result<Vec<ScheduleSuggestion>> {
        // Use Groq to analyze and generate schedule suggestions
        let prompt = self.build_suggestion_prompt(request);
        
        let client = reqwest::Client::new();
        let groq_request = serde_json::json!({
            "model": "llama3-70b-8192",
            "messages": [
                {
                    "role": "system",
                    "content": "You are a smart scheduling assistant. Analyze the user's goals and existing calendar to suggest optimal time slots. Return suggestions in JSON format."
                },
                {
                    "role": "user", 
                    "content": prompt
                }
            ],
            "temperature": 0.3,
            "max_tokens": 1000
        });
        
        let response = client
            .post("https://api.groq.com/openai/v1/chat/completions")
            .header("Authorization", "Bearer YOUR_GROQ_API_KEY")
            .header("Content-Type", "application/json")
            .json(&groq_request)
            .send()
            .await?;
        
        let groq_response: serde_json::Value = response.json().await?;
        let content = groq_response["choices"][0]["message"]["content"]
            .as_str()
            .unwrap_or("[]");
        
        // Parse the AI response
        self.parse_suggestions(content, &request.goal_description)
    }
    
    fn build_suggestion_prompt(&self, request: &ScheduleRequest) -> String {
        let events_json = serde_json::to_string(&request.existing_events).unwrap_or_default();
        let preferences_json = serde_json::to_string(&request.preferences).unwrap_or_default();
        
        format!(
            r#"Given the following information, suggest 3 optimal time slots for the user's goal:

Goal: {}
Duration: {} minutes
Existing events: {}
Time preferences: {}

Please suggest specific time slots in the next 7 days that don't conflict with existing events and match the user's preferences. 

Return the response in this exact JSON format:
{{
    "suggestions": [
        {{
            "title": "Event title",
            "description": "Detailed description",
            "start_time": "2025-01-20T10:00:00Z",
            "end_time": "2025-01-20T11:00:00Z", 
            "confidence_score": 0.85,
            "reasoning": "Why this time slot is optimal"
        }}
    ]
}}"#,
            request.goal_description,
            request.duration_minutes,
            events_json,
            preferences_json
        )
    }
    
    fn parse_suggestions(&self, ai_response: &str, goal_description: &str) -> Result<Vec<ScheduleSuggestion>> {
        let parsed: serde_json::Value = serde_json::from_str(ai_response)
            .unwrap_or_else(|_| serde_json::json!({"suggestions": []}));
        
        let suggestions = parsed["suggestions"].as_array().unwrap_or(&vec![]);
        
        let mut result = Vec::new();
        for (index, suggestion) in suggestions.iter().enumerate() {
            if let (Some(start_str), Some(end_str)) = (
                suggestion["start_time"].as_str(),
                suggestion["end_time"].as_str()
            ) {
                let start_time: DateTime<Utc> = start_time.parse()
                    .unwrap_or_else(|_| Utc::now());
                let end_time: DateTime<Utc> = end_str.parse()
                    .unwrap_or_else(|_| Utc::now() + Duration::hours(1));
                
                result.push(ScheduleSuggestion {
                    id: format!("suggestion_{}", index),
                    goal_id: "goal_temp".to_string(),
                    title: suggestion["title"].as_str().unwrap_or(goal_description).to_string(),
                    description: suggestion["description"].as_str().unwrap_or("").to_string(),
                    start_time,
                    end_time,
                    confidence_score: suggestion["confidence_score"].as_f64().unwrap_or(0.5) as f32,
                    reasoning: suggestion["reasoning"].as_str().unwrap_or("").to_string(),
                    created_at: Utc::now(),
                    status: SuggestionStatus::Pending,
                });
            }
        }
        
        Ok(result)
    }
}

pub async fn create_goal_from_natural_language(
    description: &str,
    user_id: &str,
    access_token: &str,
) -> Result<SchedulingGoal> {
    let client = reqwest::Client::new();
    
    let prompt = format!(
        r#"Parse this scheduling goal and extract structured information:

Goal: "{}"

Return in this JSON format:
{{
    "goal_type": "exercise|hobby|learning|social|work|custom",
    "custom_type": "Custom type if goal_type is 'custom'",
    "frequency": "daily|weekly|monthly|custom",
    "custom_frequency": 5,
    "duration_minutes": 60,
    "preferred_times": [
        {{
            "day_of_week": "monday|tuesday|wednesday|thursday|friday|saturday|sunday|null",
            "start_time": "09:00",
            "end_time": "17:00"
        }}
    ]
}}"#,
        description
    );
    
    let groq_request = serde_json::json!({
        "model": "llama3-70b-8192",
        "messages": [
            {
                "role": "system",
                "content": "You are a scheduling goal parser. Extract structured information from natural language goals."
            },
            {
                "role": "user",
                "content": prompt
            }
        ],
        "temperature": 0.1,
        "max_tokens": 500
    });
    
    let response = client
        .post("https://api.groq.com/openai/v1/chat/completions")
        .header("Authorization", "Bearer YOUR_GROQ_API_KEY")
        .header("Content-Type", "application/json")
        .json(&groq_request)
        .send()
        .await?;
    
    let groq_response: serde_json::Value = response.json().await?;
    let content = groq_response["choices"][0]["message"]["content"]
        .as_str()
        .unwrap_or("{}");
    
    let parsed: serde_json::Value = serde_json::from_str(content)
        .unwrap_or_else(|_| serde_json::json!({}));
    
    let goal_type_str = parsed["goal_type"].as_str().unwrap_or("custom");
    let goal_type = match goal_type_str {
        "exercise" => GoalType::Exercise,
        "hobby" => GoalType::Hobby,
        "learning" => GoalType::Learning,
        "social" => GoalType::Social,
        "work" => GoalType::Work,
        _ => GoalType::Custom(parsed["custom_type"].as_str().unwrap_or("custom").to_string()),
    };
    
    let frequency_str = parsed["frequency"].as_str().unwrap_or("weekly");
    let frequency = match frequency_str {
        "daily" => Frequency::Daily,
        "weekly" => Frequency::Weekly,
        "monthly" => Frequency::Monthly,
        _ => Frequency::Custom(parsed["custom_frequency"].as_u64().unwrap_or(1) as u32),
    };
    
    let mut preferences = Vec::new();
    if let Some(prefs_array) = parsed["preferred_times"].as_array() {
        for pref in prefs_array {
            if let (Some(start), Some(end)) = (
                pref["start_time"].as_str(),
                pref["end_time"].as_str()
            ) {
                let day_of_week = pref["day_of_week"].as_str().and_then(|d| {
                    match d {
                        "monday" => Some(Weekday::Mon),
                        "tuesday" => Some(Weekday::Tue),
                        "wednesday" => Some(Weekday::Wed),
                        "thursday" => Some(Weekday::Thu),
                        "friday" => Some(Weekday::Fri),
                        "saturday" => Some(Weekday::Sat),
                        "sunday" => Some(Weekday::Sun),
                        _ => None,
                    }
                });
                
                preferences.push(TimePreference {
                    day_of_week,
                    start_time: start.to_string(),
                    end_time: end.to_string(),
                });
            }
        }
    }
    
    Ok(SchedulingGoal {
        id: format!("goal_{}", Utc::now().timestamp()),
        user_id: user_id.to_string(),
        goal_type,
        description: description.to_string(),
        frequency,
        duration_minutes: parsed["duration_minutes"].as_u64().unwrap_or(60) as u32,
        preferred_times: preferences,
        created_at: Utc::now(),
        is_active: true,
    })
}