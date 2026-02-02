import { useState } from 'react';
import { Bell, Plus, Trash2, Edit2, Check, X, AlertTriangle } from 'lucide-react';
import type { Task, Settings } from '../types';

const COLORS = [
  '#FF6B6B', '#4ECDC4', '#45B7D1', '#96CEB4', '#FFEAA7',
  '#DDA0DD', '#98D8C8', '#F7DC6F', '#BB8FCE', '#85C1E9'
];

interface SettingsPanelProps {
  settings: Settings;
  endWorkTime: string;
  showDeleteConfirm: boolean;
  onEndWorkTimeChange: (value: string) => void;
  onSaveEndWorkTime: () => void;
  onAddTask: (name: string, color: string) => Promise<boolean>;
  onUpdateTask: (task: Task) => Promise<boolean>;
  onDeleteTask: (taskId: string) => Promise<boolean>;
  onShowDeleteConfirm: (show: boolean) => void;
  onDeleteAllData: () => Promise<boolean>;
}

export function SettingsPanel({
  settings,
  endWorkTime,
  showDeleteConfirm,
  onEndWorkTimeChange,
  onSaveEndWorkTime,
  onAddTask,
  onUpdateTask,
  onDeleteTask,
  onShowDeleteConfirm,
  onDeleteAllData
}: SettingsPanelProps) {
  const [editingTask, setEditingTask] = useState<Task | null>(null);
  const [newTaskName, setNewTaskName] = useState('');
  const [newTaskColor, setNewTaskColor] = useState('#4ECDC4');

  const handleAddTask = async () => {
    if (!newTaskName.trim()) return;
    const success = await onAddTask(newTaskName.trim(), newTaskColor);
    if (success) {
      setNewTaskName('');
    }
  };

  const handleUpdateTask = async () => {
    if (!editingTask) return;
    const success = await onUpdateTask(editingTask);
    if (success) {
      setEditingTask(null);
    }
  };

  const handleKeyDown = (e: React.KeyboardEvent<HTMLInputElement>) => {
    if (e.key === 'Enter') {
      handleAddTask();
    }
  };

  return (
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
            onChange={(e) => onEndWorkTimeChange(e.target.value)}
          />
          <button onClick={onSaveEndWorkTime}>保存</button>
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
                    {COLORS.map((c) => (
                      <button
                        key={c}
                        className={editingTask.color === c ? 'selected' : ''}
                        style={{ backgroundColor: c }}
                        onClick={() => setEditingTask({ ...editingTask, color: c })}
                      />
                    ))}
                  </div>
                  <button onClick={handleUpdateTask}><Check size={16} /></button>
                  <button onClick={() => setEditingTask(null)}><X size={16} /></button>
                </>
              ) : (
                <>
                  <span className="task-name-display">{task.name}</span>
                  <span className="shortcut-badge">
                    {task.shortcut_index ? `Ctrl+Shift+${task.shortcut_index}` : '-'}
                  </span>
                  <button onClick={() => setEditingTask(task)}><Edit2 size={16} /></button>
                  <button onClick={() => onDeleteTask(task.id)}><Trash2 size={16} /></button>
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
            onKeyDown={handleKeyDown}
          />
          <div className="color-picker">
            {COLORS.map((c) => (
              <button
                key={c}
                className={newTaskColor === c ? 'selected' : ''}
                style={{ backgroundColor: c }}
                onClick={() => setNewTaskColor(c)}
              />
            ))}
          </div>
          <button onClick={handleAddTask}>
            <Plus size={16} /> 追加
          </button>
        </div>
      </div>

      <div className="setting-item danger-zone">
        <label><AlertTriangle size={16} /> データ管理</label>
        <button className="danger-btn" onClick={() => onShowDeleteConfirm(true)}>
          <Trash2 size={16} /> 過去のデータを全削除
        </button>
      </div>

      {showDeleteConfirm && (
        <div className="modal-overlay">
          <div className="modal delete-confirm-modal">
            <div className="modal-header">
              <AlertTriangle size={24} className="warning-icon" />
              <h3>警告</h3>
            </div>
            <div className="modal-body">
              <p>すべての作業記録データが削除されます。</p>
              <p className="warning-text">この操作は取り消せません。</p>
            </div>
            <div className="modal-actions">
              <button className="cancel-btn" onClick={() => onShowDeleteConfirm(false)}>
                キャンセル
              </button>
              <button className="confirm-delete-btn" onClick={onDeleteAllData}>
                削除する
              </button>
            </div>
          </div>
        </div>
      )}
    </section>
  );
}
