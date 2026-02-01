use chrono::{DateTime, Local};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::sync::Mutex;
use tauri::menu::{Menu, MenuItem, PredefinedMenuItem};
use tauri::tray::{MouseButton, MouseButtonState, TrayIconBuilder, TrayIconEvent};
use tauri::{AppHandle, Manager, State, WindowEvent};
use tauri_plugin_clipboard_manager::ClipboardExt;
use tauri_plugin_global_shortcut::GlobalShortcutExt;
use tauri_plugin_notification::NotificationExt;

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
    pub duration_seconds: Option<i64>,
    pub details: Option<String>,
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
    eprintln!("[DEBUG] Saving daily log to: {:?}", path);
    eprintln!("[DEBUG] Data directory: {:?}", data_dir);
    eprintln!("[DEBUG] Records count: {}", log.records.len());
    let content = serde_json::to_string_pretty(log).map_err(|e| {
        eprintln!("[DEBUG] JSON serialization error: {}", e);
        e.to_string()
    })?;
    fs::write(&path, content).map_err(|e| {
        eprintln!("[DEBUG] File write error: {}", e);
        e.to_string()
    })
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
        let duration = (record.end_time.unwrap() - record.start_time).num_seconds();
        record.duration_seconds = Some(duration);
        
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
        duration_seconds: None,
        details: None,
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
        let duration = (record.end_time.unwrap() - record.start_time).num_seconds();
        record.duration_seconds = Some(duration);
        
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
        if let Some(duration) = record.duration_seconds {
            *summary.entry(record.task_name.clone()).or_insert(0) += duration;
        }
    }
    
    if let Ok(current) = state.current_record.lock() {
        if let Some(record) = current.as_ref() {
            let duration = (Local::now() - record.start_time).num_seconds();
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PeriodSummary {
    pub task_name: String,
    pub total_seconds: i64,
    pub daily_breakdown: Vec<DailyTotal>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DailyTotal {
    pub date: String,
    pub total_seconds: i64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskWithRecords {
    pub task_name: String,
    pub total_seconds: i64,
    pub records: Vec<WorkRecord>,
}

#[tauri::command]
fn get_daily_records(date: String, state: State<AppState>) -> Result<Vec<TaskWithRecords>, String> {
    let data_dir = state.data_dir.lock().map_err(|e| e.to_string())?;
    let log = load_daily_log(&data_dir, &date);
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    
    // タスクごとに記録をグループ化
    let mut task_records: HashMap<String, Vec<WorkRecord>> = HashMap::new();
    for record in &log.records {
        task_records.entry(record.task_name.clone()).or_default().push(record.clone());
    }
    
    // 設定に登録された順序で結果を作成
    let mut result: Vec<TaskWithRecords> = Vec::new();
    for task in &settings.tasks {
        if let Some(records) = task_records.get(&task.name) {
            let total_seconds: i64 = records.iter()
                .filter_map(|r| r.duration_seconds)
                .sum();
            
            if total_seconds > 0 || !records.is_empty() {
                result.push(TaskWithRecords {
                    task_name: task.name.clone(),
                    total_seconds,
                    records: records.clone(),
                });
            }
        }
    }
    
    // 進行中のタスクを追加
    if let Ok(current) = state.current_record.lock() {
        if let Some(record) = current.as_ref() {
            let duration = (Local::now() - record.start_time).num_seconds();
            
            // 既存のタスクに追加するか、新規作成
            if let Some(task_with_records) = result.iter_mut().find(|t| t.task_name == record.task_name) {
                task_with_records.total_seconds += duration;
                task_with_records.records.push(record.clone());
            } else {
                result.push(TaskWithRecords {
                    task_name: record.task_name.clone(),
                    total_seconds: duration,
                    records: vec![record.clone()],
                });
            }
        }
    }
    
    Ok(result)
}

#[tauri::command]
fn copy_summary_to_clipboard(date: String, state: State<AppState>, app: AppHandle) -> Result<String, String> {
    let data_dir = state.data_dir.lock().map_err(|e| e.to_string())?;
    let log = load_daily_log(&data_dir, &date);
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    
    // 作業ごとに記録をグループ化
    let mut task_records: HashMap<String, Vec<&WorkRecord>> = HashMap::new();
    for record in &log.records {
        task_records.entry(record.task_name.clone()).or_default().push(record);
    }
    
    let mut text = String::new();
    text.push_str(&format!("作業時間記録 ({})\n", date));
    text.push_str("=".repeat(40).as_str());
    text.push('\n');
    
    // 設定に登録された順序で作業を処理
    for task in &settings.tasks {
        if let Some(records) = task_records.get(&task.name) {
            if records.is_empty() {
                continue;
            }
            
            // 合計時間を計算
            let total_seconds: i64 = records.iter()
                .filter_map(|r| r.duration_seconds)
                .sum();
            
            if total_seconds == 0 {
                continue;
            }
            
            // 作業名と合計時間
            let hours = total_seconds / 3600;
            let mins = (total_seconds % 3600) / 60;
            let secs = total_seconds % 60;
            
            if hours > 0 {
                text.push_str(&format!("\n【{}】合計: {}時間{}分{}秒\n", task.name, hours, mins, secs));
            } else if mins > 0 {
                text.push_str(&format!("\n【{}】合計: {}分{}秒\n", task.name, mins, secs));
            } else {
                text.push_str(&format!("\n【{}】合計: {}秒\n", task.name, secs));
            }
            
            // 各記録の詳細
            for record in records {
                if let (Some(end), Some(duration)) = (record.end_time, record.duration_seconds) {
                    let start = record.start_time;
                    let start_str = start.format("%H:%M:%S");
                    let end_str = end.format("%H:%M:%S");
                    let dur_hours = duration / 3600;
                    let dur_mins = (duration % 3600) / 60;
                    let dur_secs = duration % 60;
                    
                    if dur_hours > 0 {
                        text.push_str(&format!("  {} - {} ({}時間{}分{}秒)", 
                            start_str, end_str, dur_hours, dur_mins, dur_secs));
                    } else if dur_mins > 0 {
                        text.push_str(&format!("  {} - {} ({}分{}秒)", 
                            start_str, end_str, dur_mins, dur_secs));
                    } else {
                        text.push_str(&format!("  {} - {} ({}秒)", 
                            start_str, end_str, dur_secs));
                    }
                    
                    // 詳細があれば追加
                    if let Some(details) = &record.details {
                        if !details.is_empty() {
                            text.push_str(&format!(" - {}", details));
                        }
                    }
                    text.push('\n');
                }
            }
        }
    }
    
    // 進行中の作業があれば追加
    if let Ok(current) = state.current_record.lock() {
        if let Some(record) = current.as_ref() {
            let duration = (Local::now() - record.start_time).num_seconds();
            let start_str = record.start_time.format("%H:%M:%S");
            let hours = duration / 3600;
            let mins = (duration % 3600) / 60;
            let secs = duration % 60;
            
            text.push_str(&format!("\n【{}】進行中...\n", record.task_name));
            if hours > 0 {
                text.push_str(&format!("  {} - 現在 ({}時間{}分{}秒経過)\n", 
                    start_str, hours, mins, secs));
            } else if mins > 0 {
                text.push_str(&format!("  {} - 現在 ({}分{}秒経過)\n", 
                    start_str, mins, secs));
            } else {
                text.push_str(&format!("  {} - 現在 ({}秒経過)\n", 
                    start_str, secs));
            }
        }
    }
    
    if text.lines().count() <= 2 {
        text = format!("作業時間記録 ({})\n本日の記録はありません", date);
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

#[tauri::command]
fn get_summary_by_range(
    start_date: String,
    end_date: String,
    state: State<AppState>,
) -> Result<Vec<PeriodSummary>, String> {
    let data_dir = state.data_dir.lock().map_err(|e| e.to_string())?;
    let settings = state.settings.lock().map_err(|e| e.to_string())?;
    
    // 日付範囲を生成
    let start = chrono::NaiveDate::parse_from_str(&start_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid start date: {}", e))?;
    let end = chrono::NaiveDate::parse_from_str(&end_date, "%Y-%m-%d")
        .map_err(|e| format!("Invalid end date: {}", e))?;
    
    let mut task_daily_totals: HashMap<String, HashMap<String, i64>> = HashMap::new();
    
    // 各日のログを読み込んで集計
    let mut current = start;
    while current <= end {
        let date_str = current.format("%Y-%m-%d").to_string();
        let log = load_daily_log(&data_dir, &date_str);
        
        for record in &log.records {
            if let Some(duration) = record.duration_seconds {
                let daily_map = task_daily_totals
                    .entry(record.task_name.clone())
                    .or_default();
                *daily_map.entry(date_str.clone()).or_insert(0) += duration;
            }
        }
        
        current = current.succ_opt().unwrap_or(current);
    }
    
    // 進行中のタスクを追加
    if let Ok(current_record) = state.current_record.lock() {
        if let Some(record) = current_record.as_ref() {
            let today = Local::now().format("%Y-%m-%d").to_string();
            let today_date = chrono::NaiveDate::parse_from_str(&today, "%Y-%m-%d").unwrap();
            
            if today_date >= start && today_date <= end {
                let duration = (Local::now() - record.start_time).num_seconds();
                let daily_map = task_daily_totals
                    .entry(record.task_name.clone())
                    .or_default();
                *daily_map.entry(today).or_insert(0) += duration;
            }
        }
    }
    
    // 結果を構築
    let mut result: Vec<PeriodSummary> = Vec::new();
    for task in &settings.tasks {
        if let Some(daily_map) = task_daily_totals.get(&task.name) {
            let total_seconds: i64 = daily_map.values().sum();
            if total_seconds > 0 {
                let mut daily_breakdown: Vec<DailyTotal> = daily_map
                    .iter()
                    .map(|(date, &seconds)| DailyTotal {
                        date: date.clone(),
                        total_seconds: seconds,
                    })
                    .collect();
                daily_breakdown.sort_by(|a, b| a.date.cmp(&b.date));
                
                result.push(PeriodSummary {
                    task_name: task.name.clone(),
                    total_seconds,
                    daily_breakdown,
                });
            }
        }
    }
    
    Ok(result)
}

#[tauri::command]
fn copy_period_summary_to_clipboard(
    start_date: String,
    end_date: String,
    state: State<AppState>,
    app: AppHandle,
) -> Result<String, String> {
    let summary = get_summary_by_range(start_date.clone(), end_date.clone(), state)?;
    
    let mut text = String::new();
    text.push_str(&format!("作業時間記録 ({} ~ {})\n", start_date, end_date));
    text.push_str("=".repeat(40).as_str());
    text.push('\n');
    
    let total_all: i64 = summary.iter().map(|s| s.total_seconds).sum();
    let total_hours = total_all / 3600;
    let total_mins = (total_all % 3600) / 60;
    
    if total_hours > 0 {
        text.push_str(&format!("\n合計: {}時間{}分\n", total_hours, total_mins));
    } else {
        text.push_str(&format!("\n合計: {}分\n", total_mins));
    }
    
    for task_summary in &summary {
        let hours = task_summary.total_seconds / 3600;
        let mins = (task_summary.total_seconds % 3600) / 60;
        
        if hours > 0 {
            text.push_str(&format!("\n【{}】{}時間{}分\n", task_summary.task_name, hours, mins));
        } else {
            text.push_str(&format!("\n【{}】{}分\n", task_summary.task_name, mins));
        }
        
        for daily in &task_summary.daily_breakdown {
            let d_hours = daily.total_seconds / 3600;
            let d_mins = (daily.total_seconds % 3600) / 60;
            let d_secs = daily.total_seconds % 60;
            
            if d_hours > 0 {
                text.push_str(&format!("  {}: {}時間{}分{}秒\n", daily.date, d_hours, d_mins, d_secs));
            } else if d_mins > 0 {
                text.push_str(&format!("  {}: {}分{}秒\n", daily.date, d_mins, d_secs));
            } else {
                text.push_str(&format!("  {}: {}秒\n", daily.date, d_secs));
            }
        }
    }
    
    if summary.is_empty() {
        text.push_str("\nこの期間の記録はありません");
    }
    
    app.clipboard().write_text(text.clone()).map_err(|e| e.to_string())?;
    Ok(text)
}

#[tauri::command]
fn update_record_details(
    date: String,
    record_id: String,
    details: Option<String>,
    state: State<AppState>,
) -> Result<(), String> {
    let data_dir = state.data_dir.lock().map_err(|e| e.to_string())?;
    let mut log = load_daily_log(&data_dir, &date);
    
    // 該当する記録を検索して更新
    if let Some(record) = log.records.iter_mut().find(|r| r.id == record_id) {
        record.details = details;
        save_daily_log(&data_dir, &log)?;
        Ok(())
    } else {
        Err("Record not found".to_string())
    }
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
                let app_handle = app.clone();
                std::thread::spawn(move || {
                    let state = app_handle.state::<AppState>();
                    let _ = stop_task(state);
                });
            }
            "restart" => {
                let app_handle = app.clone();
                std::thread::spawn(move || {
                    let state = app_handle.state::<AppState>();
                    let task_id = {
                        let last_id = state.last_task_id.lock();
                        last_id.ok().and_then(|guard| guard.clone())
                    };
                    if let Some(id) = task_id {
                        let state = app_handle.state::<AppState>();
                        let _ = start_task(id, state);
                    }
                });
            }
            "quit" => {
                let app_handle = app.clone();
                std::thread::spawn(move || {
                    let state = app_handle.state::<AppState>();
                    let _ = stop_task(state);
                    app_handle.exit(0);
                });
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
    use tauri_plugin_global_shortcut::Shortcut;
    
    // Ctrl+Shift+S - 停止
    let shortcut_s: Shortcut = "Ctrl+Shift+S".parse()?;
    let app_handle = app.clone();
    app.global_shortcut().on_shortcut(shortcut_s, move |_app, _shortcut, _event| {
        let state = app_handle.state::<AppState>();
        let _ = stop_task(state);
    })?;
    
    // Ctrl+Shift+R - 再開
    let shortcut_r: Shortcut = "Ctrl+Shift+R".parse()?;
    let app_handle = app.clone();
    app.global_shortcut().on_shortcut(shortcut_r, move |_app, _shortcut, _event| {
        let state = app_handle.state::<AppState>();
        let task_id = {
            let last_id = state.last_task_id.lock();
            last_id.ok().and_then(|guard| guard.clone())
        };
        if let Some(id) = task_id {
            let state = app_handle.state::<AppState>();
            let _ = start_task(id, state);
        }
    })?;
    
    // Ctrl+Shift+1~9 - 作業切替
    for i in 1..=9 {
        let shortcut: Shortcut = format!("Ctrl+Shift+{}", i).parse()?;
        let app_handle = app.clone();
        app.global_shortcut().on_shortcut(shortcut, move |_app, _shortcut, _event| {
            let state = app_handle.state::<AppState>();
            let task_id = {
                let settings = state.settings.lock();
                settings.ok().and_then(|s| {
                    s.tasks.iter().find(|t| t.shortcut_index == Some(i)).map(|t| t.id.clone())
                })
            };
            if let Some(id) = task_id {
                let state = app_handle.state::<AppState>();
                let _ = start_task(id, state);
            }
        })?;
    }
    
    Ok(())
}

fn setup_reminder(app: &AppHandle) {
    let app_handle = app.clone();
    
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(60));
            
            let state = app_handle.state::<AppState>();
            let (reminder_enabled, end_work_time) = {
                let settings = state.settings.lock();
                if let Ok(s) = settings {
                    (s.reminder_enabled, s.end_work_time.clone())
                } else {
                    continue;
                }
            };
            
            if !reminder_enabled {
                continue;
            }
            
            let now = Local::now();
            let current_time = now.format("%H:%M").to_string();
            
            if current_time == end_work_time {
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
    });
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_opener::init())
        .plugin(tauri_plugin_global_shortcut::Builder::new().build())
        .plugin(tauri_plugin_clipboard_manager::init())
        .plugin(tauri_plugin_notification::init())
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
            get_daily_records,
            copy_summary_to_clipboard,
            add_task,
            update_task,
            delete_task,
            update_record_details,
            get_summary_by_range,
            copy_period_summary_to_clipboard
        ])
        .run(tauri::generate_context!())
        .expect("error while running tauri application");
}
