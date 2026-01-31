import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import { 
  Play, 
  Square, 
  Settings, 
  Plus, 
  Trash2, 
  Edit2, 
  Clock, 
  Copy,
  Check,
  X,
  Bell
} from 'lucide-react';
import './App.css';
import type { Task, WorkRecord, Settings as SettingsType } from './types';

function App() {
  const [settings, setSettings] = useState<SettingsType | null>(null);
  const [currentTask, setCurrentTask] = useState<WorkRecord | null>(null);
  const [dailySummary, setDailySummary] = useState<[string, number][]>([]);
  const [elapsedTime, setElapsedTime] = useState<number>(0);
  const [showSettings, setShowSettings] = useState(false);
  const [editingTask, setEditingTask] = useState<Task | null>(null);
  const [newTaskName, setNewTaskName] = useState('');
  const [newTaskColor, setNewTaskColor] = useState('#4ECDC4');
  const [copied, setCopied] = useState(false);
  const [endWorkTime, setEndWorkTime] = useState('19:00');

  const loadSettings = useCallback(async () => {
    try {
      const s = await invoke<SettingsType>('get_settings');
      setSettings(s);
      setEndWorkTime(s.end_work_time);
    } catch (e) {
      console.error('Failed to load settings:', e);
    }
  }, []);

  const loadCurrentTask = useCallback(async () => {
    try {
      const task = await invoke<WorkRecord | null>('get_current_task');
      setCurrentTask(task);
      if (task) {
        const start = new Date(task.start_time).getTime();
        setElapsedTime(Math.floor((Date.now() - start) / 1000));
      } else {
        setElapsedTime(0);
      }
    } catch (e) {
      console.error('Failed to load current task:', e);
    }
  }, []);

  const loadDailySummary = useCallback(async () => {
    try {
      const today = new Date().toISOString().split('T')[0];
      const summary = await invoke<[string, number][]>('get_daily_summary', { date: today });
      setDailySummary(summary);
    } catch (e) {
      console.error('Failed to load daily summary:', e);
    }
  }, []);

  useEffect(() => {
    loadSettings();
    loadCurrentTask();
    loadDailySummary();

    const interval = setInterval(() => {
      loadCurrentTask();
      loadDailySummary();
    }, 5000);

    return () => clearInterval(interval);
  }, [loadSettings, loadCurrentTask, loadDailySummary]);

  useEffect(() => {
    if (!currentTask) return;

    const interval = setInterval(() => {
      const start = new Date(currentTask.start_time).getTime();
      setElapsedTime(Math.floor((Date.now() - start) / 1000));
    }, 1000);

    return () => clearInterval(interval);
  }, [currentTask]);

  const startTask = async (taskId: string) => {
    try {
      await invoke('start_task', { taskId });
      await loadCurrentTask();
      await loadDailySummary();
    } catch (e) {
      console.error('Failed to start task:', e);
    }
  };

  const stopTask = async () => {
    try {
      await invoke('stop_task');
      setCurrentTask(null);
      setElapsedTime(0);
      await loadDailySummary();
    } catch (e) {
      console.error('Failed to stop task:', e);
    }
  };

  const copySummary = async () => {
    try {
      const today = new Date().toISOString().split('T')[0];
      await invoke<string>('copy_summary_to_clipboard', { date: today });
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    } catch (e) {
      console.error('Failed to copy summary:', e);
    }
  };

  const addTask = async () => {
    if (!newTaskName.trim()) return;
    try {
      await invoke('add_task', { 
        name: newTaskName.trim(), 
        color: newTaskColor 
      });
      setNewTaskName('');
      await loadSettings();
    } catch (e) {
      console.error('Failed to add task:', e);
    }
  };

  const updateTask = async () => {
    if (!editingTask) return;
    try {
      await invoke('update_task', { task: editingTask });
      setEditingTask(null);
      await loadSettings();
    } catch (e) {
      console.error('Failed to update task:', e);
    }
  };

  const deleteTask = async (taskId: string) => {
    if (!confirm('この作業を削除しますか？')) return;
    try {
      await invoke('delete_task', { taskId });
      await loadSettings();
    } catch (e) {
      console.error('Failed to delete task:', e);
    }
  };

  const saveEndWorkTime = async () => {
    try {
      if (settings) {
        const newSettings = { ...settings, end_work_time: endWorkTime };
        await invoke('save_settings_command', { settings: newSettings });
        setSettings(newSettings);
      }
    } catch (e) {
      console.error('Failed to save end work time:', e);
    }
  };

  const formatTime = (seconds: number) => {
    const hours = Math.floor(seconds / 3600);
    const mins = Math.floor((seconds % 3600) / 60);
    const secs = seconds % 60;
    return `${hours.toString().padStart(2, '0')}:${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
  };

  const formatDuration = (minutes: number) => {
    const hours = Math.floor(minutes / 60);
    const mins = minutes % 60;
    if (hours > 0) {
      return `${hours}時間${mins}分`;
    }
    return `${mins}分`;
  };

  const colors = [
    '#FF6B6B', '#4ECDC4', '#45B7D1', '#96CEB4', '#FFEAA7',
    '#DDA0DD', '#98D8C8', '#F7DC6F', '#BB8FCE', '#85C1E9'
  ];

  if (!settings) {
    return <div className="loading">読み込み中...</div>;
  }

  return (
    <div className="app">
      <header className="header">
        <h1>作業時間記録</h1>
        <button className="icon-btn" onClick={() => setShowSettings(!showSettings)}>
          <Settings size={20} />
        </button>
      </header>

      <section className="current-task">
        <div className="timer-display">
          <Clock size={24} />
          <span className="timer">{formatTime(elapsedTime)}</span>
        </div>
        {currentTask ? (
          <div className="active-task">
            <span className="task-name">{currentTask.task_name}</span>
            <button className="stop-btn" onClick={stopTask}>
              <Square size={18} /> 停止
            </button>
          </div>
        ) : (
          <div className="no-task">作業を選択してください</div>
        )}
      </section>

      {!showSettings && (
        <section className="task-list">
          <h2>作業一覧</h2>
          <div className="tasks">
            {settings.tasks.map((task) => (
              <button
                key={task.id}
                className={`task-btn ${currentTask?.task_id === task.id ? 'active' : ''}`}
                style={{ 
                  borderLeftColor: task.color || '#4ECDC4',
                  backgroundColor: currentTask?.task_id === task.id ? (task.color || '#4ECDC4') + '20' : undefined
                }}
                onClick={() => startTask(task.id)}
              >
                <span className="task-index">{task.shortcut_index ? `Ctrl+Shift+${task.shortcut_index}` : ''}</span>
                <span className="task-name">{task.name}</span>
                <Play size={16} className="play-icon" />
              </button>
            ))}
          </div>
        </section>
      )}

      {showSettings && (
        <section className="settings">
          <h2>設定</h2>
          
          <div className="setting-item">
            <label>
              <Bell size={16} /> 終業時刻リマインダー
            </label>
            <div className="time-input">
              <input
                type="time"
                value={endWorkTime}
                onChange={(e) => setEndWorkTime(e.target.value)}
              />
              <button onClick={saveEndWorkTime}>保存</button>
            </div>
          </div>

          <div className="setting-item">
            <label>作業一覧</label>
            <div className="task-editor">
              {settings.tasks.map((task) => (
                <div key={task.id} className="task-edit-row">
                  {editingTask?.id === task.id ? (
                    <>
                      <input
                        type="text"
                        value={editingTask.name}
                        onChange={(e) => setEditingTask({ ...editingTask, name: e.target.value })}
                      />
                      <div className="color-picker">
                        {colors.map((c) => (
                          <button
                            key={c}
                            className={editingTask.color === c ? 'selected' : ''}
                            style={{ backgroundColor: c }}
                            onClick={() => setEditingTask({ ...editingTask, color: c })}
                          />
                        ))}
                      </div>
                      <button onClick={updateTask}><Check size={16} /></button>
                      <button onClick={() => setEditingTask(null)}><X size={16} /></button>
                    </>
                  ) : (
                    <>
                      <span style={{ borderLeft: `4px solid ${task.color || '#4ECDC4'}`, paddingLeft: 8 }}>
                        {task.name}
                      </span>
                      <span className="shortcut-badge">
                        {task.shortcut_index ? `Ctrl+Shift+${task.shortcut_index}` : '-'}
                      </span>
                      <button onClick={() => setEditingTask(task)}><Edit2 size={16} /></button>
                      <button onClick={() => deleteTask(task.id)}><Trash2 size={16} /></button>
                    </>
                  )}
                </div>
              ))}
            </div>
          </div>

          <div className="setting-item">
            <label>新規作業追加</label>
            <div className="add-task-form">
              <input
                type="text"
                placeholder="作業名"
                value={newTaskName}
                onChange={(e) => setNewTaskName(e.target.value)}
                onKeyPress={(e) => e.key === 'Enter' && addTask()}
              />
              <div className="color-picker">
                {colors.map((c) => (
                  <button
                    key={c}
                    className={newTaskColor === c ? 'selected' : ''}
                    style={{ backgroundColor: c }}
                    onClick={() => setNewTaskColor(c)}
                  />
                ))}
              </div>
              <button onClick={addTask}>
                <Plus size={16} /> 追加
              </button>
            </div>
          </div>
        </section>
      )}

      <section className="daily-summary">
        <div className="summary-header">
          <h2>本日の記録</h2>
          <button className={`copy-btn ${copied ? 'copied' : ''}`} onClick={copySummary}>
            {copied ? <><Check size={16} /> コピー完了</> : <><Copy size={16} /> コピー</>}
          </button>
        </div>
        <div className="summary-list">
          {dailySummary.filter(([, mins]) => mins > 0).length === 0 ? (
            <div className="no-records">本日の記録はありません</div>
          ) : (
            dailySummary
              .filter(([, mins]) => mins > 0)
              .map(([name, minutes]) => (
                <div key={name} className="summary-item">
                  <span className="name">{name}</span>
                  <span className="time">{formatDuration(minutes)}</span>
                </div>
              ))
          )}
        </div>
      </section>

      <footer className="shortcuts">
        <div className="shortcut">
          <kbd>Ctrl+Shift+S</kbd>
          <span>停止</span>
        </div>
        <div className="shortcut">
          <kbd>Ctrl+Shift+R</kbd>
          <span>再開</span>
        </div>
        <div className="shortcut">
          <kbd>Ctrl+Shift+1~9</kbd>
          <span>作業切替</span>
        </div>
      </footer>
    </div>
  );
}

export default App;
