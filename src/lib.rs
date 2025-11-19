use jni::JNIEnv;
use jni::objects::{JClass, JString, JObject, JValue};
use jni::sys::{jstring, jobject, jint};
use android_logger::Config;
use log::LevelFilter;
use std::sync::Mutex;

mod auth;
mod calendar;
mod file_handler;
mod groq;
mod storage;
mod overlay;
mod drive;
mod scheduler;

static APP_STATE: Mutex<Option<app_state::AppState>> = Mutex::new(None);

mod app_state {
    use super::*;
    
    pub struct AppState {
        pub google_token: Option<String>,
        pub selected_calendars: Vec<String>,
        pub calendar_users: Vec<String>,
        pub sparka_folder_id: Option<String>,
        pub db: storage::Database,
        pub scheduling_goals: Vec<scheduler::SchedulingGoal>,
        pub suggestions: Vec<scheduler::ScheduleSuggestion>,
    }
    
    impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            google_token: None,
            selected_calendars: Vec::new(),
            calendar_users: Vec::new(),
            sparka_folder_id: None,
            db: storage::Database::new()?,
            scheduling_goals: Vec::new(),
            suggestions: Vec::new(),
        })
    }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_sparka_SparkaApplication_init(
    env: JNIEnv,
    _class: JClass,
    context: JObject,
) {
    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Debug)
            .with_tag("Sparka"),
    );
    
    log::info!("Initializing Sparka app");
    
    let mut state = APP_STATE.lock().unwrap();
    *state = Some(app_state::AppState::new().expect("Failed to initialize app state"));
}

#[no_mangle]
pub extern "system" fn Java_com_sparka_MainActivity_handleSharedFile(
    env: JNIEnv,
    _class: JClass,
    file_uri: JString,
) -> jstring {
    let file_uri_str: String = env.get_string(file_uri).expect("Invalid string");
    
    log::info!("Handling shared file: {}", file_uri_str);
    
    match handle_shared_file_internal(file_uri_str) {
        Ok(result) => {
            let result_str = env.new_string(result).expect("Failed to create string");
            result_str.into_raw()
        }
        Err(e) => {
            log::error!("Error handling shared file: {}", e);
            let error_str = env.new_string(format!("Error: {}", e)).expect("Failed to create string");
            error_str.into_raw()
        }
    }
}

fn handle_shared_file_internal(file_uri: String) -> anyhow::Result<String> {
    let rt = tokio::runtime::Runtime::new()?;
    rt.block_on(async {
        handle_shared_file_internal_async(file_uri).await
    })
}

async fn handle_shared_file_internal_async(file_uri: String) -> anyhow::Result<String> {
    let mut state = APP_STATE.lock().unwrap();
    let app_state = state.as_mut().ok_or_else(|| anyhow::anyhow!("App not initialized"))?;
    
    // Ensure Sparka folder exists
    if app_state.sparka_folder_id.is_none() {
        let folder_id = drive::create_or_get_sparka_folder(&app_state.google_token.as_ref().unwrap()).await?;
        app_state.sparka_folder_id = Some(folder_id);
    }
    
    // Read file content
    let content = file_handler::read_file_from_uri(&file_uri)?;
    let file_name = file_handler::extract_filename_from_uri(&file_uri)?;
    let mime_type = file_handler::get_mime_type(&file_uri)?;
    
    // Upload to Drive
    let drive_file = drive::upload_file_to_drive(
        &app_state.google_token.as_ref().unwrap(),
        &app_state.sparka_folder_id.as_ref().unwrap(),
        &file_name,
        content.as_bytes(),
        &mime_type,
    ).await?;
    
    // Share with calendar users
    if !app_state.calendar_users.is_empty() {
        drive::share_folder_with_calendar_users(
            &app_state.google_token.as_ref().unwrap(),
            &app_state.sparka_folder_id.as_ref().unwrap(),
            &app_state.calendar_users,
        ).await?;
    }
    
    // Analyze with Groq
    let analysis = groq::analyze_file(&content, &app_state.google_token).await?;
    
    // Create calendar event
    let event_id = calendar::create_event(
        &app_state.google_token.as_ref().unwrap(),
        &app_state.selected_calendars[0], // Use first selected calendar
        &analysis,
    ).await?;
    
    // Store ticket data offline
    storage::store_ticket(&app_state.db, &event_id, &analysis, &content)?;
    
    // Add import record to Drive log
    drive::add_import_record(
        &app_state.google_token.as_ref().unwrap(),
        &app_state.sparka_folder_id.as_ref().unwrap(),
        &drive_file.id,
        &drive_file.name,
        "current_user", // TODO: Get actual user email
        Some(event_id.clone()),
        "imported",
        None,
    ).await?;
    
    Ok(format!("Ticket processed, uploaded to Drive, and calendar event created: {}", event_id))
}

#[no_mangle]
pub extern "system" fn Java_com_sparka_MainActivity_storeGoogleToken(
    env: JNIEnv,
    _class: JClass,
    access_token: JString,
) -> jstring {
    let access_token_str: String = env.get_string(access_token).expect("Invalid string");
    
    match auth::store_access_token(&access_token_str) {
        Ok(_) => {
            // Store token in app state
            let mut state = APP_STATE.lock().unwrap();
            if let Some(ref mut app_state) = *state {
                app_state.google_token = Some(access_token_str.clone());
            }
            
            let success_str = env.new_string("Google Calendar connected successfully").expect("Failed to create string");
            success_str.into_raw()
        }
        Err(e) => {
            let error_str = env.new_string(format!("Auth error: {}", e)).expect("Failed to create string");
            error_str.into_raw()
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_sparka_MainActivity_getCalendars(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let state = APP_STATE.lock().unwrap();
    let app_state = state.as_ref().ok_or_else(|| anyhow::anyhow!("App not initialized"));
    
    match app_state.and_then(|s| calendar::list_calendars(&s.google_token)) {
        Ok(calendars) => {
            let calendars_json = serde_json::to_string(&calendars).expect("Failed to serialize calendars");
            let calendars_str = env.new_string(calendars_json).expect("Failed to create string");
            calendars_str.into_raw()
        }
        Err(e) => {
            let error_str = env.new_string(format!("Error: {}", e)).expect("Failed to create string");
            error_str.into_raw()
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_sparka_MainActivity_setSelectedCalendars(
    env: JNIEnv,
    _class: JClass,
    calendars_json: JString,
) {
    let calendars_str: String = env.get_string(calendars_json).expect("Invalid string");
    
    match serde_json::from_str::<Vec<String>>(&calendars_str) {
        Ok(calendars) => {
            let mut state = APP_STATE.lock().unwrap();
            if let Some(ref mut app_state) = *state {
                app_state.selected_calendars = calendars;
            }
        }
        Err(e) => {
            log::error!("Error parsing calendars: {}", e);
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_sparka_MainActivity_setCalendarUsers(
    env: JNIEnv,
    _class: JClass,
    users_json: JString,
) {
    let users_str: String = env.get_string(users_json).expect("Invalid string");
    
    match serde_json::from_str::<Vec<String>>(&users_str) {
        Ok(users) => {
            let mut state = APP_STATE.lock().unwrap();
            if let Some(ref mut app_state) = *state {
                app_state.calendar_users = users;
            }
        }
        Err(e) => {
            log::error!("Error parsing calendar users: {}", e);
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_sparka_MainActivity_shareWithUser(
    env: JNIEnv,
    _class: JClass,
    email: JString,
) -> jstring {
    let email_str: String = env.get_string(email).expect("Invalid string");
    
    let rt = tokio::runtime::Runtime::new();
    let result = match rt {
        Ok(rt) => {
            rt.block_on(async {
                let state = APP_STATE.lock().unwrap();
                let app_state = state.as_ref().ok_or_else(|| anyhow::anyhow!("App not initialized"));
                
                match app_state.and_then(|s| {
                    match s.sparka_folder_id {
                        Some(ref folder_id) => {
                            drive::share_file_with_user(&s.google_token.as_ref().unwrap(), folder_id, &email_str, "reader").await
                        }
                        None => Err(anyhow::anyhow!("Sparka folder not created yet"))
                    }
                }) {
                    Ok(_) => Ok("File shared successfully".to_string()),
                    Err(e) => Ok(format!("Failed to share: {}", e))
                }
            })
        }
        Err(e) => Ok(format!("Runtime error: {}", e))
    };
    
    match result {
        Ok(msg) => {
            let msg_str = env.new_string(msg).expect("Failed to create string");
            msg_str.into_raw()
        }
        Err(_) => {
            let error_str = env.new_string("Unknown error occurred".to_string()).expect("Failed to create string");
            error_str.into_raw()
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_sparka_MainActivity_getDriveFiles(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let rt = tokio::runtime::Runtime::new();
    let result = match rt {
        Ok(rt) => {
            rt.block_on(async {
                let state = APP_STATE.lock().unwrap();
                let app_state = state.as_ref().ok_or_else(|| anyhow::anyhow!("App not initialized"));
                
                match app_state.and_then(|s| {
                    match s.sparka_folder_id {
                        Some(ref folder_id) => {
                            drive::list_files_in_folder(&s.google_token.as_ref().unwrap(), folder_id).await
                        }
                        None => Err(anyhow::anyhow!("Sparka folder not created yet"))
                    }
                }) {
                    Ok(files) => {
                        let files_json = serde_json::to_string(&files).expect("Failed to serialize files");
                        Ok(files_json)
                    }
                    Err(e) => Ok(format!("Error: {}", e))
                }
            })
        }
        Err(e) => Ok(format!("Runtime error: {}", e))
    };
    
    match result {
        Ok(msg) => {
            let msg_str = env.new_string(msg).expect("Failed to create string");
            msg_str.into_raw()
        }
        Err(_) => {
            let error_str = env.new_string("Unknown error occurred".to_string()).expect("Failed to create string");
            error_str.into_raw()
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_sparka_MainActivity_createSchedulingGoal(
    env: JNIEnv,
    _class: JClass,
    goal_description: JString,
    user_id: JString,
) -> jstring {
    let goal_desc_str: String = env.get_string(goal_description).expect("Invalid string");
    let user_id_str: String = env.get_string(user_id).expect("Invalid string");
    
    let rt = tokio::runtime::Runtime::new();
    let result = match rt {
        Ok(rt) => {
            rt.block_on(async {
                let mut state = APP_STATE.lock().unwrap();
                let app_state = state.as_mut().ok_or_else(|| anyhow::anyhow!("App not initialized"));
                
                match app_state {
                    Some(ref mut s) => {
                        match scheduler::create_goal_from_natural_language(
                            &goal_desc_str,
                            &user_id_str,
                            &s.google_token.as_ref().unwrap(),
                        ).await {
                            Ok(goal) => {
                                // Store goal in database
                                if let Err(e) = storage::store_scheduling_goal(&s.db, &goal) {
                                    Ok(format!("Error storing goal: {}", e))
                                } else {
                                    s.scheduling_goals.push(goal);
                                    Ok("Goal created successfully".to_string())
                                }
                            }
                            Err(e) => Ok(format!("Error creating goal: {}", e))
                        }
                    }
                    None => Ok("App not initialized".to_string())
                }
            })
        }
        Err(e) => Ok(format!("Runtime error: {}", e))
    };
    
    match result {
        Ok(msg) => {
            let msg_str = env.new_string(msg).expect("Failed to create string");
            msg_str.into_raw()
        }
        Err(_) => {
            let error_str = env.new_string("Unknown error occurred".to_string()).expect("Failed to create string");
            error_str.into_raw()
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_sparka_MainActivity_generateScheduleSuggestions(
    env: JNIEnv,
    _class: JClass,
    user_id: JString,
) -> jstring {
    let user_id_str: String = env.get_string(user_id).expect("Invalid string");
    
    let rt = tokio::runtime::Runtime::new();
    let result = match rt {
        Ok(rt) => {
            rt.block_on(async {
                let state = APP_STATE.lock().unwrap();
                let app_state = state.as_ref().ok_or_else(|| anyhow::anyhow!("App not initialized"));
                
                match app_state {
                    Some(s) => {
                        // Get active goals
                        match storage::get_active_goals(&s.db, &user_id_str) {
                            Ok(goals) => {
                                let mut all_suggestions = Vec::new();
                                
                                for goal in goals {
                                    // Get calendar events for next 7 days
                                    match calendar::get_events_for_period(
                                        &s.google_token.as_ref().unwrap(),
                                        &s.selected_calendars[0],
                                        chrono::Utc::now(),
                                        chrono::Utc::now() + chrono::Duration::days(7),
                                    ).await {
                                        Ok(events) => {
                                            let request = scheduler::ScheduleRequest {
                                                goal_description: goal.description.clone(),
                                                existing_events: events,
                                                preferences: goal.preferred_times.clone(),
                                                duration_minutes: goal.duration_minutes,
                                            };
                                            
                                            let engine = scheduler::SuggestionEngine::new();
                                            match engine.generate_suggestions(&request, &s.google_token.as_ref().unwrap()).await {
                                                Ok(mut suggestions) => {
                                                    // Store suggestions in database
                                                    for suggestion in &suggestions {
                                                        let _ = storage::store_schedule_suggestion(&s.db, suggestion);
                                                    }
                                                    all_suggestions.append(&mut suggestions);
                                                }
                                                Err(e) => log::error!("Error generating suggestions: {}", e),
                                            }
                                        }
                                        Err(e) => log::error!("Error getting calendar events: {}", e),
                                    }
                                }
                                
                                let suggestions_json = serde_json::to_string(&all_suggestions)
                                    .expect("Failed to serialize suggestions");
                                Ok(suggestions_json)
                            }
                            Err(e) => Ok(format!("Error getting goals: {}", e))
                        }
                    }
                    None => Ok("App not initialized".to_string())
                }
            })
        }
        Err(e) => Ok(format!("Runtime error: {}", e))
    };
    
    match result {
        Ok(msg) => {
            let msg_str = env.new_string(msg).expect("Failed to create string");
            msg_str.into_raw()
        }
        Err(_) => {
            let error_str = env.new_string("Unknown error occurred".to_string()).expect("Failed to create string");
            error_str.into_raw()
        }
    }
}

#[no_mangle]
pub extern "system" fn Java_com_sparka_SuggestionReceiver_acceptSuggestion(
    env: JNIEnv,
    _class: JClass,
    suggestion_id: JString,
    title: JString,
    start_time: JString,
) {
    let suggestion_id_str: String = env.get_string(suggestion_id).expect("Invalid string");
    let title_str: String = env.get_string(title).expect("Invalid string");
    let start_time_str: String = env.get_string(start_time).expect("Invalid string");
    
    let rt = tokio::runtime::Runtime::new();
    let _ = rt.block_on(async {
        let state = APP_STATE.lock().unwrap();
        let app_state = state.as_ref().ok_or_else(|| anyhow::anyhow!("App not initialized"));
        
        match app_state {
            Some(s) => {
                // Create calendar event from suggestion
                let event = calendar::CalendarEvent {
                    id: "".to_string(),
                    summary: format!("🎯 {}", title_str),
                    description: "AI-generated schedule suggestion".to_string(),
                    start: calendar::EventTime {
                        date_time: Some(start_time_str.parse::<chrono::DateTime<chrono::Utc>>().unwrap_or_else(|_| chrono::Utc::now())),
                        date: None,
                    },
                    end: calendar::EventTime {
                        date_time: Some(start_time_str.parse::<chrono::DateTime<chrono::Utc>>().unwrap_or_else(|_| chrono::Utc::now()) + chrono::Duration::hours(1)),
                        date: None,
                    },
                };
                
                // Update suggestion status
                let _ = storage::update_suggestion_status(&s.db, &suggestion_id_str, scheduler::SuggestionStatus::Accepted);
                
                log::info!("Accepted suggestion: {} - {}", suggestion_id_str, title_str);
            }
            None => log::error!("App not initialized"),
        }
    });
}

#[no_mangle]
pub extern "system" fn Java_com_sparka_SuggestionReceiver_rejectSuggestion(
    env: JNIEnv,
    _class: JClass,
    suggestion_id: JString,
) {
    let suggestion_id_str: String = env.get_string(suggestion_id).expect("Invalid string");
    
    let rt = tokio::runtime::Runtime::new();
    let _ = rt.block_on(async {
        let state = APP_STATE.lock().unwrap();
        let app_state = state.as_ref().ok_or_else(|| anyhow::anyhow!("App not initialized"));
        
        match app_state {
            Some(s) => {
                // Update suggestion status
                let _ = storage::update_suggestion_status(&s.db, &suggestion_id_str, scheduler::SuggestionStatus::Rejected);
                log::info!("Rejected suggestion: {}", suggestion_id_str);
            }
            None => log::error!("App not initialized"),
        }
    });
}

#[no_mangle]
pub extern "system" fn Java_com_sparka_MainActivity_init(
    env: JNIEnv,
    _class: JClass,
    context: JObject,
) {
    android_logger::init_once(
        Config::default()
            .with_max_level(LevelFilter::Debug)
            .with_tag("Sparka"),
    );
    
    log::info!("Initializing Sparka app");
    
    let mut state = APP_STATE.lock().unwrap();
    *state = Some(app_state::AppState::new().expect("Failed to initialize app state"));
}

#[no_mangle]
pub extern "system" fn Java_com_sparka_OverlayService_checkAndShowOverlays(
    env: JNIEnv,
    _class: JClass,
) -> jint {
    let state = APP_STATE.lock().unwrap();
    let app_state = state.as_ref().ok_or_else(|| anyhow::anyhow!("App not initialized"));
    
    match app_state.and_then(|s| storage::get_active_tickets(&s.db)) {
        Ok(tickets) => {
            let mut active_count = 0;
            for ticket in tickets {
                if overlay::should_show_overlay(&ticket) {
                    // Trigger overlay display via Android system
                    active_count += 1;
                }
            }
            active_count as jint
        }
        Err(_) => 0
    }
}

#[no_mangle]
pub extern "system" fn Java_com_sparka_OverlayService_getActiveTicketInfo(
    env: JNIEnv,
    _class: JClass,
) -> jstring {
    let state = APP_STATE.lock().unwrap();
    let app_state = state.as_ref().ok_or_else(|| anyhow::anyhow!("App not initialized"));
    
    match app_state.and_then(|s| storage::get_active_tickets(&s.db)) {
        Ok(tickets) => {
            if let Some(ticket) = tickets.first() {
                match overlay::format_overlay_text(ticket) {
                    Ok(text) => {
                        let text_str = env.new_string(text).expect("Failed to create string");
                        text_str.into_raw()
                    }
                    Err(_) => {
                        let error_str = env.new_string("Error formatting ticket".to_string()).expect("Failed to create string");
                        error_str.into_raw()
                    }
                }
            } else {
                let empty_str = env.new_string("".to_string()).expect("Failed to create string");
                empty_str.into_raw()
            }
        }
        Err(_) => {
            let error_str = env.new_string("Error getting tickets".to_string()).expect("Failed to create string");
            error_str.into_raw()
        }
    }
}