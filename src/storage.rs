use anyhow::Result;
use rusqlite::{Connection, params, Row};
use crate::calendar::TicketAnalysis;
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