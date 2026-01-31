export interface Task {
  id: string;
  name: string;
  color?: string;
  shortcut_index?: number;
}

export interface WorkRecord {
  id: string;
  task_id: string;
  task_name: string;
  start_time: string;
  end_time?: string;
  duration_minutes?: number;
}

export interface Settings {
  tasks: Task[];
  end_work_time: string;
  reminder_enabled: boolean;
}
