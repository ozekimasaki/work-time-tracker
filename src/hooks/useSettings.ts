import { useState, useEffect, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { Settings, Task } from '../types';

export function useSettings() {
  const [settings, setSettings] = useState<Settings | null>(null);
  const [isLoading, setIsLoading] = useState(true);
  const [error, setError] = useState<Error | null>(null);

  const loadSettings = useCallback(async () => {
    try {
      setIsLoading(true);
      const s = await invoke<Settings>('get_settings');
      setSettings(s);
      setError(null);
      return s;
    } catch (e) {
      const err = e instanceof Error ? e : new Error(String(e));
      setError(err);
      console.error('Failed to load settings:', e);
      return null;
    } finally {
      setIsLoading(false);
    }
  }, []);

  const saveSettings = useCallback(async (newSettings: Settings) => {
    try {
      await invoke('save_settings_command', { settings: newSettings });
      setSettings(newSettings);
      return true;
    } catch (e) {
      console.error('Failed to save settings:', e);
      return false;
    }
  }, []);

  const addTask = useCallback(async (name: string, color: string) => {
    try {
      await invoke('add_task', { name: name.trim(), color });
      await loadSettings();
      return true;
    } catch (e) {
      console.error('Failed to add task:', e);
      return false;
    }
  }, [loadSettings]);

  const updateTask = useCallback(async (task: Task) => {
    try {
      await invoke('update_task', { task });
      await loadSettings();
      return true;
    } catch (e) {
      console.error('Failed to update task:', e);
      return false;
    }
  }, [loadSettings]);

  const deleteTask = useCallback(async (taskId: string) => {
    if (!confirm('この作業を削除しますか？')) return false;
    try {
      await invoke('delete_task', { task_id: taskId });
      await loadSettings();
      return true;
    } catch (e) {
      console.error('Failed to delete task:', e);
      return false;
    }
  }, [loadSettings]);

  const deleteAllData = useCallback(async () => {
    try {
      await invoke('delete_all_data');
      await loadSettings();
      return true;
    } catch (e) {
      console.error('Failed to delete all data:', e);
      return false;
    }
  }, [loadSettings]);

  useEffect(() => {
    loadSettings();
  }, [loadSettings]);

  return {
    settings,
    isLoading,
    error,
    loadSettings,
    saveSettings,
    addTask,
    updateTask,
    deleteTask,
    deleteAllData
  };
}
