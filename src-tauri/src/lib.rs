use chrono::{DateTime, Local, NaiveTime};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::notification::NotificationExt;
use tauri::{AppHandle, Manager, State, Window, WindowEvent};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::{GlobalShortcutExt, ShortcutState};

// データ構造
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub shortcut_index: Option<u8>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkRecord {
    pub id: String,
    pub task_id: String,
    pub task_name: String,
    pub start_time: DateTime<Local>,
    pub end_time: Option<DateTime<Local>>,
    pub duration_minutes: Option<i64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyLog {
    pub date: String,
    pub records: Vec<WorkRecord>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Settings {
    pub tasks: Vec<Task>,
    pub end_work_time: String,
    pub reminder_enabled: bool,
}

impl Default for Settings {
    fn default() -> Self {
        Settings {
            tasks: vec![
                Task { id: "1".to_string(), name: "会議".to_string(), color: Some("#FF6B6B".to_string()), shortcut_index: Some(1) },
                Task { id: "2".to_string(), name: "コーディング".to_string(), color: Some("#4ECDC4".to_string()), shortcut_index: Some(2) },
                Task { id: "3".to_string(), name: "資料作成".to_string(), color: Some("#45B7D1".to_string()), shortcut_index: Some(3) },
                Task { id: "4".to_string(), name: "メール対応".to_string(), color: Some("#96CEB4".to_string()), shortcut_index: Some(4) },
                Task { id: "5".to_string(), name: "調査・研究".to_string(), color: Some("#FFEAA7".to_string()), shortcut_index: Some(5) },
            ],
            end_work_time: "19:00".to_string(),
            reminder_enabled: true,
        }
    }
}

pub struct AppState {
    pub settings: Mutex<Settings>,
    pub current_record: Mutex<Option<WorkRecord>>,
    pub last_task_id: Mutex<Option<String>>,
    pub data_dir: Mutex<PathBuf>,
}

fn get_data_dir(app_handle: &AppHandle) -> PathBuf {
    let mut path = app_handle
        .path()
        .app_data_dir()
        .expect("Failed to get app data dir");
    path.push("work-time-tracker");
    fs::create_dir_all(&path).ok();
    path
}

fn get_settings_path(data_dir: &PathBuf) -> PathBuf {
    data_dir.join("settings.json")
}

fn get_daily_log_path(data_dir: &PathBuf, date: &str) -> PathBuf {
    data_dir.join(format!("{}.json", date))
}

fn load_settings(data_dir: &PathBuf) -> Settings {
    let path = get_settings_path(data_dir);
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(settings) = serde_json::from_str(&content) {
            return settings;
        }
    }
    Settings::default()
}

fn save_settings(data_dir: &PathBuf, settings: &Settings) -> Result<(), String> {
    let path = get_settings_path(data_dir);
    let content = serde_json::to_string_pretty(settings).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}

fn load_daily_log(data_dir: &PathBuf, date: &str) -> DailyLog {
    let path = get_daily_log_path(data_dir, date);
    if let Ok(content) = fs::read_to_string(&path) {
        if let Ok(log) = serde_json::from_str(&content) {
            return log;
        }
    }
    DailyLog {
        date: date.to_string(),
        records: vec![],
    }
}

fn save_daily_log(data_dir: &PathBuf, log: &DailyLog) -> Result<(), String> {
    let path = get_daily_log_path(data_dir, &log.date);
    let content = serde_json::to_string_pretty(log).map_err(|e| e.to_string())?;
    fs::write(&path, content).map_err(|e| e.to_string())
}

#[tauri::command]
fn get_settings(state: State<AppState>) -> Result<Settings, String> {
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    Ok(settings.clone())
}

#[tauri::command]
fn save_settings_command(
    settings: Settings,
    state: State<AppState>,
) -> Result<(), String> {
    let data_dir = state.data_dir.lock().map_err(|e| e.to_string())?;
    save_settings(&data_dir, &settings)?;
    let mut state_settings = state.settings.lock().map_err(|e| e.to_string())?;
    *state_settings = settings;
    Ok(())
}

#[tauri::command]
fn start_task(
    task_id: String,
    state: State<AppState>,
) -> Result<WorkRecord, String> {
    let data_dir = state.data_dir.lock().map_err(|e| e.to_string())?;
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    
    let mut current = state.current_record.lock().map_err(|e| e.to_string())?;
    if let Some(mut record) = current.take() {
        record.end_time = Some(Local::now());
        let duration = (record.end_time.unwrap() - record.start_time).num_minutes();
        record.duration_minutes = Some(duration);
        
        let today = Local::now().format("%Y-%m-%d").to_string();
        let mut log = load_daily_log(&data_dir, &today);
        log.records.push(record);
        save_daily_log(&data_dir, &log)?;
    }
    
    let task_name = settings
        .tasks
        .iter()
        .find(|t| t.id == task_id)
        .map(|t| t.name.clone())
        .unwrap_or_else(|| "不明".to_string());
    
    let new_record = WorkRecord {
        id: format!("{}", chrono::Local::now().timestamp_millis()),
        task_id: task_id.clone(),
        task_name,
        start_time: Local::now(),
        end_time: None,
        duration_minutes: None,
    };
    
    *current = Some(new_record.clone());
    
    let mut last = state.last_task_id.lock().map_err(|e| e.to_string())?;
    *last = Some(task_id);
    
    Ok(new_record)
}

#[tauri::command]
fn stop_task(state: State<AppState>) -> Result<Option<WorkRecord>, String> {
    let data_dir = state.data_dir.lock().map_err(|e| e.to_string())?;
    let mut current = state.current_record.lock().map_err(|e| e.to_string())?;
    
    if let Some(mut record) = current.take() {
        record.end_time = Some(Local::now());
        let duration = (record.end_time.unwrap() - record.start_time).num_minutes();
        record.duration_minutes = Some(duration);
        
        let today = Local::now().format("%Y-%m-%d").to_string();
        let mut log = load_daily_log(&data_dir, &today);
        log.records.push(record.clone());
        save_daily_log(&data_dir, &log)?;
        
        Ok(Some(record))
    } else {
        Ok(None)
    }
}

#[tauri::command]
fn get_current_task(state: State<AppState>) -> Result<Option<WorkRecord>, String> {
    let current = state.current_record.lock().map_err(|e| e.to_string())?;
    Ok(current.clone())
}

#[tauri::command]
fn get_daily_summary(date: String, state: State<AppState>) -> Result<Vec<(String, i64)>, String> {
    let data_dir = state.data_dir.lock().map_err(|e| e.to_string())?;
    let log = load_daily_log(&data_dir, &date);
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    
    let mut summary: HashMap<String, i64> = HashMap::new();
    
    for task in &settings.tasks {
        summary.insert(task.name.clone(), 0);
    }
    
    for record in &log.records {
        if let Some(duration) = record.duration_minutes {
            *summary.entry(record.task_name.clone()).or_insert(0) += duration;
        }
    }
    
    if let Ok(current) = state.current_record.lock() {
        if let Some(record) = current.as_ref() {
            let duration = (Local::now() - record.start_time).num_minutes();
            *summary.entry(record.task_name.clone()).or_insert(0) += duration;
        }
    }
    
    let result: Vec<(String, i64)> = settings
        .tasks
        .iter()
        .filter_map(|t| {
            summary.get(&t.name).map(|&mins| (t.name.clone(), mins))
        })
        .collect();
    
    Ok(result)
}

#[tauri::command]
fn copy_summary_to_clipboard(date: String, state: State<AppState>, app: AppHandle) -> Result<String, String> {
    let summary = get_daily_summary(date, state)?;
    
    let mut text = String::new();
    for (task_name, minutes) in summary {
        if minutes > 0 {
            let hours = minutes / 60;
            let mins = minutes % 60;
            if hours > 0 {
                text.push_str(&format!("{}: {}時間{}分\n", task_name, hours, mins));
            } else {
                text.push_str(&format!("{}: {}分\n", task_name, mins));
            }
        }
    }
    
    if text.is_empty() {
        text = "本日の記録はありません".to_string();
    }
    
    app.clipboard().write_text(text.clone()).map_err(|e| e.to_string())?;
    Ok(text)
}

#[tauri::command]
fn add_task(name: String, color: Option<String>, state: State<AppState>) -> Result<Task, String> {
    let data_dir = state.data_dir.lock().map_err(|e| e.to_string())?;
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    
    let used_indices: Vec<u8> = settings
        .tasks
        .iter()
        .filter_map(|t| t.shortcut_index)
        .collect();
    
    let shortcut_index = (1..=9).find(|i| !used_indices.contains(i));
    
    let task = Task {
        id: format!("{}", chrono::Local::now().timestamp_millis()),
        name,
        color,
        shortcut_index,
    };
    
    settings.tasks.push(task.clone());
    save_settings(&data_dir, &settings)?;
    
    Ok(task)
}

#[tauri::command]
fn update_task(task: Task, state: State<AppState>) -> Result<(), String> {
    let data_dir = state.data_dir.lock().map_err(|e| e.to_string())?;
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    
    if let Some(existing) = settings.tasks.iter_mut().find(|t| t.id == task.id) {
        *existing = task;
        save_settings(&data_dir, &settings)?;
    }
    
    Ok(())
}

#[tauri::command]
fn delete_task(task_id: String, state: State<AppState>) -> Result<(), String> {
    let data_dir = state.data_dir.lock().map_err(|e| e.to_string())?;
    let mut settings = state.settings.lock().map_err(|e| e.to_string())?;
    
    settings.tasks.retain(|t| t.id != task_id);
    save_settings(&data_dir, &settings)?;
    
    Ok(())
}

fn setup_tray(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    let menu = Menu::with_items(
        app,
        &[
            &MenuItem::with_id(app, "show", "表示", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "stop", "停止 (Ctrl+Shift+S)", true, None::<&str>)?,
            &MenuItem::with_id(app, "restart", "再開 (Ctrl+Shift+R)", true, None::<&str>)?,
            &PredefinedMenuItem::separator(app)?,
            &MenuItem::with_id(app, "quit", "終了", true, None::<&str>)?,
        ],
    )?;

    TrayIconBuilder::new()
        .icon(app.default_window_icon().unwrap().clone())
        .tooltip("作業時間記録")
        .menu(&menu)
        .on_menu_event(|app, event| match event.id.as_ref() {
            "show" => {
                if let Some(window) = app.get_webview_window("main") {
                    window.show().ok();
                    window.set_focus().ok();
                }
            }
            "stop" => {
                let state = app.state::<AppState>();
                let _ = stop_task(state);
            }
            "restart" => {
                let state = app.state::<AppState>();
                if let Ok(last_id) = state.last_task_id.lock() {
                    if let Some(id) = last_id.clone() {
                        let _ = start_task(id, state);
                    }
                }
            }
            "quit" => {
                let state = app.state::<AppState>();
                let _ = stop_task(state);
                app.exit(0);
            }
            _ => {}
        })
        .on_tray_icon_event(|tray, event| {
            if let TrayIconEvent::Click { button_state, button, .. } = event {
                if button == MouseButton::Left && button_state == MouseButtonState::Up {
                    if let Some(window) = tray.app_handle().get_webview_window("main") {
                        window.show().ok();
                        window.set_focus().ok();
                    }
                }
            }
        })
        .build(app)?;

    Ok(())
}

fn setup_global_shortcuts(app: &AppHandle) -> Result<(), Box<dyn std::error::Error>> {
    app.global_shortcut().register("Ctrl+Shift+S")?;
    app.global_shortcut().register("Ctrl+Shift+R")?;
    
    for i in 1..=9 {
        app.global_shortcut().register(&format!("Ctrl+Shift+{}", i))?;
    }
    
    let app_handle = app.clone();
    app.global_shortcut().on_shortcut("Ctrl+Shift+S", move |_, _| {
        let state = app_handle.state::<AppState>();
        let _ = stop_task(state);
    });
    
    let app_handle = app.clone();
    app.global_shortcut().on_shortcut("Ctrl+Shift+R", move |_, _| {
        let state = app_handle.state::<AppState>();
        if let Ok(last_id) = state.last_task_id.lock() {
            if let Some(id) = last_id.clone() {
                let _ = start_task(id, state);
            }
        }
    });
    
    for i in 1..=9 {
        let app_handle = app.clone();
        let shortcut = format!("Ctrl+Shift+{}", i);
        app.global_shortcut().on_shortcut(&shortcut, move |_, _| {
            let state = app_handle.state::<AppState>();
            if let Ok(settings) = state.settings.lock() {
                if let Some(task) = settings.tasks.iter().find(|t| t.shortcut_index == Some(i)) {
                    let _ = start_task(task.id.clone(), state);
                }
            }
        });
    }
    
    Ok(())
}

fn setup_reminder(app: &AppHandle) {
    let app_handle = app.clone();
    
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(60));
            
            let state = app_handle.state::<AppState>();
            if let Ok(settings) = state.settings.lock() {
                if !settings.reminder_enabled {
                    continue;
                }
                
                let now = Local::now();
                let current_time = now.format("%H:%M").to_string();
                
                if current_time == settings.end_work_time {
                    let _ = app_handle
                        .notification()
                        .builder()
                        .title("終業時間です")
                        .body("本日の作業時間を確認してください")
                        .show();
                    
                    if let Some(window) = app_handle.get_webview_window("main") {
                        let _ = window.show();
                        let _ = window.set_focus();
                    }
                }
            }
        }
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .manage(AppState {
            settings: Mutex::new(Settings::default()),
            current_record: Mutex::new(None),
            last_task_id: Mutex::new(None),
            data_dir: Mutex::new(PathBuf::new()),
        })
        .setup(|app| {
            let app_handle = app.handle();
            let data_dir = get_data_dir(&app_handle);
            
            {
                let state = app.state::<AppState>();
                let mut dir = state.data_dir.lock().unwrap();
                *dir = data_dir.clone();
            }
            
            {
                let state = app.state::<AppState>();
                let settings = load_settings(&data_dir);
                let mut state_settings = state.settings.lock().unwrap();
                *state_settings = settings;
            }
            
            setup_tray(&app_handle)?;
            setup_global_shortcuts(&app_handle)?;
            setup_reminder(&app_handle);
            
            if let Some(window) = app_handle.get_webview_window("main") {
                let window_handle = window.clone();
                window.on_window_event(move |event| {
                    if let WindowEvent::CloseRequested { api, .. } = event {
                        api.prevent_close();
                        window_handle.hide().ok();
                    }
                });
            }
            
            Ok(())
        })
        .invoke_handler(tauri::generate_handler![
            get_settings,
            save_settings_command,
            start_task,
            stop_task,
            get_current_task,
            get_daily_summary,
            copy_summary_to_clipboard,
            add_task,
            update_task,
            delete_task
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
