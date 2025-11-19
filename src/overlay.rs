use crate::storage::TicketRecord;
use chrono::{DateTime, Utc, Duration};
use anyhow::Result;

pub fn should_show_overlay(ticket: &TicketRecord) -> bool {
    // Parse the event date
    let event_datetime = match ticket.get_event_datetime() {
        Ok(dt) => dt,
        Err(_) => return false,
    };
    
    let now = Utc::now();
    let one_hour = Duration::hours(1);
    
    // Show overlay 1 hour before until 1 hour after the event
    let start_showing = event_datetime - one_hour;
    let stop_showing = event_datetime + one_hour;
    
    now >= start_showing && now <= stop_showing
}

pub fn format_overlay_text(ticket: &TicketRecord) -> Result<String> {
    let analysis = ticket.parse_analysis()?;
    
    Ok(format!(
        "🎫 {}\n📍 {}\n⏰ {} {}",
        analysis.event_name,
        analysis.venue,
        analysis.event_date,
        analysis.event_time
    ))
}

pub fn get_overlay_priority(ticket: &TicketRecord) -> i32 {
    let event_datetime = match ticket.get_event_datetime() {
        Ok(dt) => dt,
        Err(_) => return 0,
    };
    
    let now = Utc::now();
    let minutes_until = (event_datetime - now).num_minutes();
    
    // Higher priority for events that are closer
    if minutes_until < 0 {
        100 // Event has started, high priority
    } else if minutes_until < 30 {
        90 // Less than 30 minutes, very high priority
    } else if minutes_until < 60 {
        80 // Less than 1 hour, high priority
    } else {
        50 // More than 1 hour, normal priority
    }
}