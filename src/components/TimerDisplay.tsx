import { Clock, Square } from 'lucide-react';
import { formatTime } from '../utils/timeFormat';
import type { WorkRecord } from '../types';

interface TimerDisplayProps {
  elapsedTime: number;
  currentTask: WorkRecord | null;
  onStopTask: () => void;
}

export function TimerDisplay({ elapsedTime, currentTask, onStopTask }: TimerDisplayProps) {
  return (
    <section className="current-task">
      <div className="timer-display">
        <Clock size={24} />
        <span className="timer">{formatTime(elapsedTime)}</span>
      </div>
      {currentTask ? (
        <div className="active-task">
          <span className="task-name">{currentTask.task_name}</span>
          <button className="stop-btn" onClick={onStopTask}>
            <Square size={18} /> 停止
          </button>
        </div>
      ) : (
        <div className="no-task">作業を選択してください</div>
      )}
    </section>
  );
}
