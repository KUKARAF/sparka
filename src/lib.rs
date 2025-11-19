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

static APP_STATE: Mutex<Option<app_state::AppState>> = Mutex::new(None);

mod app_state {
    use super::*;
    
    pub struct AppState {
        pub google_token: Option<String>,
        pub selected_calendars: Vec<String>,
        pub calendar_users: Vec<String>,
        pub sparka_folder_id: Option<String>,
        pub db: storage::Database,
    }
    
    impl AppState {
    pub fn new() -> anyhow::Result<Self> {
        Ok(Self {
            google_token: None,
            selected_calendars: Vec::new(),
            calendar_users: Vec::new(),
            sparka_folder_id: None,
            db: storage::Database::new()?,
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