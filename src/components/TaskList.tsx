import { Play } from 'lucide-react';
import type { Task, WorkRecord } from '../types';

interface TaskListProps {
  tasks: Task[];
  currentTask: WorkRecord | null;
  onStartTask: (taskId: string) => void;
}

export function TaskList({ tasks, currentTask, onStartTask }: TaskListProps) {
  return (
    <section className="task-list">
      <h2>作業一覧</h2>
      <div className="tasks">
        {tasks.map((task) => (
          <button
            key={task.id}
            className={`task-btn ${currentTask?.task_id === task.id ? 'active' : ''}`}
            onClick={() => onStartTask(task.id)}
          >
            <span className="task-index">{task.shortcut_index ? `Ctrl+Shift+${task.shortcut_index}` : ''}</span>
            <span className="task-name">{task.name}</span>
            <Play size={16} className="play-icon" />
          </button>
        ))}
      </div>
    </section>
  );
}
