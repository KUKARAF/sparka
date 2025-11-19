use anyhow::Result;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use chrono::{DateTime, Utc, TimeZone};

#[derive(Debug, Serialize, Deserialize)]
pub struct Calendar {
    pub id: String,
    pub summary: String,
    pub description: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CalendarEvent {
    pub id: String,
    pub summary: String,
    pub description: Option<String>,
    pub start: EventTime,
    pub end: EventTime,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct EventTime {
    pub date_time: Option<DateTime<Utc>>,
    pub date: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct TicketAnalysis {
    pub event_name: String,
    pub event_date: String,
    pub event_time: String,
    pub venue: String,
    pub seat_info: Option<String>,
    pub ticket_type: Option<String>,
}

pub async fn list_calendars(access_token: &str) -> Result<Vec<Calendar>> {
    let client = Client::new();
    
    let response = client
        .get("https://www.googleapis.com/calendar/v3/users/me/calendarList")
        .header("Authorization", format!("Bearer {}", access_token))
        .send()
        .await?;
    
    let calendars_response: CalendarListResponse = response.json().await?;
    Ok(calendars_response.items)
}

pub async fn get_calendar_users(access_token: &str, calendar_id: &str) -> Result<Vec<String>> {
    let client = Client::new();
    
    let response = client
        .get(&format!("https://www.googleapis.com/calendar/v3/calendars/{}/acl", calendar_id))
        .header("Authorization", format!("Bearer {}", access_token))
        .query(&[("maxResults", "250")])
        .send()
        .await?;
    
    let acl_response: AclResponse = response.json().await?;
    
    let mut users = Vec::new();
    for rule in acl_response.items {
        if let Some(scope) = rule.scope {
            if scope.type_ == "user" && scope.value.is_some() {
                users.push(scope.value.unwrap());
            }
        }
    }
    
    Ok(users)
}

#[derive(Debug, Serialize, Deserialize)]
struct CalendarListResponse {
    items: Vec<Calendar>,
}

pub async fn create_event(
    access_token: &str,
    calendar_id: &str,
    analysis: &TicketAnalysis,
) -> Result<String> {
    let client = Client::new();
    
    // Parse date and time from analysis
    let start_datetime = format!("{}T{}:00", analysis.event_date, analysis.event_time);
    let end_datetime = format!("{}T{}:00", analysis.event_date, analysis.event_time);
    
    let event = GoogleCalendarEvent {
        summary: format!("🎫 {}", analysis.event_name),
        description: format!(
            "Venue: {}\nSeat: {}\nType: {}",
            analysis.venue,
            analysis.seat_info.as_deref().unwrap_or("N/A"),
            analysis.ticket_type.as_deref().unwrap_or("N/A")
        ),
        start: EventTimePayload {
            date_time: Some(start_datetime),
            time_zone: Some("UTC".to_string()),
        },
        end: EventTimePayload {
            date_time: Some(end_datetime),
            time_zone: Some("UTC".to_string()),
        },
    };
    
    let response = client
        .post(&format!(
            "https://www.googleapis.com/calendar/v3/calendars/{}/events",
            calendar_id
        ))
        .header("Authorization", format!("Bearer {}", access_token))
        .json(&event)
        .send()
        .await?;
    
    let created_event: CalendarEvent = response.json().await?;
    Ok(created_event.id)
}

#[derive(Debug, Serialize)]
struct GoogleCalendarEvent {
    summary: String,
    description: String,
    start: EventTimePayload,
    end: EventTimePayload,
}

#[derive(Debug, Serialize)]
struct EventTimePayload {
    date_time: Option<String>,
    time_zone: Option<String>,
}

#[derive(Debug, Deserialize)]
pub struct AclResponse {
    items: Vec<AclRule>,
}

#[derive(Debug, Deserialize)]
pub struct AclRule {
    pub id: String,
    pub scope: Option<AclScope>,
    pub role: String,
}

#[derive(Debug, Deserialize)]
pub struct AclScope {
    #[serde(rename = "type")]
    pub type_: String,
    pub value: Option<String>,
}

pub async fn get_events_for_period(
    access_token: &str,
    calendar_id: &str,
    start_time: chrono::DateTime<chrono::Utc>,
    end_time: chrono::DateTime<chrono::Utc>,
) -> Result<Vec<CalendarEvent>> {
    let client = Client::new();
    
    let response = client
        .get(&format!("https://www.googleapis.com/calendar/v3/calendars/{}/events", calendar_id))
        .header("Authorization", format!("Bearer {}", access_token))
        .query(&[
            ("timeMin", &start_time.to_rfc3339()),
            ("timeMax", &end_time.to_rfc3339()),
            ("singleEvents", "true"),
            ("orderBy", "startTime"),
        ])
        .send()
        .await?;
    
    let events_response: EventsResponse = response.json().await?;
    Ok(events_response.items)
}

#[derive(Debug, Deserialize)]
pub struct EventsResponse {
    items: Vec<CalendarEvent>,
}