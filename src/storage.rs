use anyhow::Result;
use rusqlite::{Connection, params, Row};
use crate::calendar::TicketAnalysis;
use crate::scheduler::{SchedulingGoal, ScheduleSuggestion};
use chrono::{DateTime, Utc};
use serde_json;

pub struct Database {
    conn: Connection,
}

impl Database {
    pub fn new() -> Result<Self> {
        let conn = Connection::open("sparka.db")?;
        
        // Initialize tables
        conn.execute(
            "CREATE TABLE IF NOT EXISTS tickets (
                id INTEGER PRIMARY KEY AUTOINCREMENT,
                event_id TEXT NOT NULL,
                analysis TEXT NOT NULL,
                raw_content TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                event_date DATETIME NOT NULL
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS user_settings (
                key TEXT PRIMARY KEY,
                value TEXT NOT NULL
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS scheduling_goals (
                id TEXT PRIMARY KEY,
                user_id TEXT NOT NULL,
                goal_type TEXT NOT NULL,
                description TEXT NOT NULL,
                frequency TEXT NOT NULL,
                duration_minutes INTEGER NOT NULL,
                preferred_times TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                is_active BOOLEAN DEFAULT 1
            )",
            [],
        )?;
        
        conn.execute(
            "CREATE TABLE IF NOT EXISTS schedule_suggestions (
                id TEXT PRIMARY KEY,
                goal_id TEXT NOT NULL,
                title TEXT NOT NULL,
                description TEXT NOT NULL,
                start_time DATETIME NOT NULL,
                end_time DATETIME NOT NULL,
                confidence_score REAL NOT NULL,
                reasoning TEXT NOT NULL,
                created_at DATETIME DEFAULT CURRENT_TIMESTAMP,
                status TEXT DEFAULT 'pending'
            )",
            [],
        )?;
        
        Ok(Database { conn })
    }
    
    pub fn store_ticket(&self, event_id: &str, analysis: &TicketAnalysis, raw_content: &str) -> Result<()> {
        let analysis_json = serde_json::to_string(analysis)?;
        let event_datetime = format!("{}T{}:00", analysis.event_date, analysis.event_time);
        
        self.conn.execute(
            "INSERT INTO tickets (event_id, analysis, raw_content, event_date) VALUES (?1, ?2, ?3, ?4)",
            params![event_id, analysis_json, raw_content, event_datetime],
        )?;
        
        Ok(())
    }
    
    pub fn get_active_tickets(&self) -> Result<Vec<TicketRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, event_id, analysis, raw_content, created_at, event_date 
             FROM tickets 
             WHERE event_date >= datetime('now', '-1 hour') 
             AND event_date <= datetime('now', '+1 hour')"
        )?;
        
        let ticket_iter = stmt.query_map([], |row| {
            Ok(TicketRecord {
                id: row.get(0)?,
                event_id: row.get(1)?,
                analysis: row.get::<_, String>(2)?,
                raw_content: row.get(3)?,
                created_at: row.get(4)?,
                event_date: row.get(5)?,
            })
        })?;
        
        let mut tickets = Vec::new();
        for ticket in ticket_iter {
            tickets.push(ticket?);
        }
        
        Ok(tickets)
    }
    
    pub fn get_ticket_by_event_id(&self, event_id: &str) -> Result<Option<TicketRecord>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, event_id, analysis, raw_content, created_at, event_date 
             FROM tickets WHERE event_id = ?1"
        )?;
        
        let ticket_iter = stmt.query_map([event_id], |row| {
            Ok(TicketRecord {
                id: row.get(0)?,
                event_id: row.get(1)?,
                analysis: row.get::<_, String>(2)?,
                raw_content: row.get(3)?,
                created_at: row.get(4)?,
                event_date: row.get(5)?,
            })
        })?;
        
        for ticket in ticket_iter {
            return Ok(Some(ticket?));
        }
        
        Ok(None)
    }
    
    pub fn set_setting(&self, key: &str, value: &str) -> Result<()> {
        self.conn.execute(
            "INSERT OR REPLACE INTO user_settings (key, value) VALUES (?1, ?2)",
            params![key, value],
        )?;
        Ok(())
    }
    
    pub fn get_setting(&self, key: &str) -> Result<Option<String>> {
        let mut stmt = self.conn.prepare("SELECT value FROM user_settings WHERE key = ?1")?;
        let mut rows = stmt.query_map([key], |row| row.get(0))?;
        
        if let Some(row) = rows.next() {
            Ok(Some(row?))
        } else {
            Ok(None)
        }
    }
    
    pub fn store_scheduling_goal(&self, goal: &SchedulingGoal) -> Result<()> {
        let goal_type_json = serde_json::to_string(&goal.goal_type)?;
        let preferences_json = serde_json::to_string(&goal.preferred_times)?;
        let frequency_json = serde_json::to_string(&goal.frequency)?;
        
        self.conn.execute(
            "INSERT OR REPLACE INTO scheduling_goals 
             (id, user_id, goal_type, description, frequency, duration_minutes, preferred_times, created_at, is_active)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                goal.id,
                goal.user_id,
                goal_type_json,
                goal.description,
                frequency_json,
                goal.duration_minutes,
                preferences_json,
                goal.created_at.format("%Y-%m-%d %H:%M:%S"),
                goal.is_active
            ],
        )?;
        Ok(())
    }
    
    pub fn get_active_goals(&self, user_id: &str) -> Result<Vec<SchedulingGoal>> {
        let mut stmt = self.conn.prepare(
            "SELECT id, user_id, goal_type, description, frequency, duration_minutes, preferred_times, created_at, is_active
             FROM scheduling_goals WHERE user_id = ?1 AND is_active = 1"
        )?;
        
        let goal_iter = stmt.query_map([user_id], |row| {
            let goal_type_str: String = row.get(2)?;
            let goal_type: scheduler::GoalType = serde_json::from_str(&goal_type_str)?;
            
            let frequency_str: String = row.get(4)?;
            let frequency: scheduler::Frequency = serde_json::from_str(&frequency_str)?;
            
            let preferences_str: String = row.get(6)?;
            let preferred_times: Vec<scheduler::TimePreference> = serde_json::from_str(&preferences_str)?;
            
            Ok(SchedulingGoal {
                id: row.get(0)?,
                user_id: row.get(1)?,
                goal_type,
                description: row.get(3)?,
                frequency,
                duration_minutes: row.get(5)?,
                preferred_times,
                created_at: row.get::<_, String>(7)?.parse::<DateTime<Utc>>()?,
                is_active: row.get(8)?,
            })
        })?;
        
        let mut goals = Vec::new();
        for goal in goal_iter {
            goals.push(goal?);
        }
        Ok(goals)
    }
    
    pub fn store_schedule_suggestion(&self, suggestion: &ScheduleSuggestion) -> Result<()> {
        let status_str = format!("{:?}", suggestion.status);
        
        self.conn.execute(
            "INSERT OR REPLACE INTO schedule_suggestions 
             (id, goal_id, title, description, start_time, end_time, confidence_score, reasoning, created_at, status)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                suggestion.id,
                suggestion.goal_id,
                suggestion.title,
                suggestion.description,
                suggestion.start_time.format("%Y-%m-%d %H:%M:%S"),
                suggestion.end_time.format("%Y-%m-%d %H:%M:%S"),
                suggestion.confidence_score,
                suggestion.reasoning,
                suggestion.created_at.format("%Y-%m-%d %H:%M:%S"),
                status_str
            ],
        )?;
        Ok(())
    }
    
    pub fn get_pending_suggestions(&self, user_id: &str) -> Result<Vec<ScheduleSuggestion>> {
        let mut stmt = self.conn.prepare(
            "SELECT s.id, s.goal_id, s.title, s.description, s.start_time, s.end_time, s.confidence_score, s.reasoning, s.created_at, s.status
             FROM schedule_suggestions s
             JOIN scheduling_goals g ON s.goal_id = g.id
             WHERE g.user_id = ?1 AND s.status = 'pending'
             ORDER BY s.confidence_score DESC"
        )?;
        
        let suggestion_iter = stmt.query_map([user_id], |row| {
            let status_str: String = row.get(9)?;
            let status = match status_str.as_str() {
                "Pending" => scheduler::SuggestionStatus::Pending,
                "Accepted" => scheduler::SuggestionStatus::Accepted,
                "Rejected" => scheduler::SuggestionStatus::Rejected,
                "Expired" => scheduler::SuggestionStatus::Expired,
                _ => scheduler::SuggestionStatus::Pending,
            };
            
            Ok(ScheduleSuggestion {
                id: row.get(0)?,
                goal_id: row.get(1)?,
                title: row.get(2)?,
                description: row.get(3)?,
                start_time: row.get::<_, String>(4)?.parse::<DateTime<Utc>>()?,
                end_time: row.get::<_, String>(5)?.parse::<DateTime<Utc>>()?,
                confidence_score: row.get(6)?,
                reasoning: row.get(7)?,
                created_at: row.get::<_, String>(8)?.parse::<DateTime<Utc>>()?,
                status,
            })
        })?;
        
        let mut suggestions = Vec::new();
        for suggestion in suggestion_iter {
            suggestions.push(suggestion?);
        }
        Ok(suggestions)
    }
    
    pub fn update_suggestion_status(&self, suggestion_id: &str, status: scheduler::SuggestionStatus) -> Result<()> {
        let status_str = format!("{:?}", status);
        self.conn.execute(
            "UPDATE schedule_suggestions SET status = ?1 WHERE id = ?2",
            params![status_str, suggestion_id],
        )?;
        Ok(())
    }
}

#[derive(Debug)]
pub struct TicketRecord {
    pub id: i64,
    pub event_id: String,
    pub analysis: String,
    pub raw_content: String,
    pub created_at: String,
    pub event_date: String,
}

impl TicketRecord {
    pub fn parse_analysis(&self) -> Result<TicketAnalysis> {
        Ok(serde_json::from_str(&self.analysis)?)
    }
    
    pub fn get_event_datetime(&self) -> Result<DateTime<Utc>> {
        Ok(self.event_date.parse::<DateTime<Utc>>()?)
    }
}