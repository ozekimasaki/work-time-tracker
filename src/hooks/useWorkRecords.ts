import { useState, useCallback } from 'react';
import { invoke } from '@tauri-apps/api/core';
import type { WorkRecord, TaskWithRecords, PeriodSummary } from '../types';
import type { DateRange } from '../utils/dateRange';

export function useWorkRecords() {
  const [currentTask, setCurrentTask] = useState<WorkRecord | null>(null);
  const [dailyRecords, setDailyRecords] = useState<TaskWithRecords[]>([]);
  const [periodSummary, setPeriodSummary] = useState<PeriodSummary[]>([]);

  const loadCurrentTask = useCallback(async () => {
    try {
      const task = await invoke<WorkRecord | null>('get_current_task');
      setCurrentTask(task);
      return task;
    } catch (e) {
      console.error('Failed to load current task:', e);
      return null;
    }
  }, []);

  const loadDailyRecords = useCallback(async (date: string) => {
    try {
      const records = await invoke<TaskWithRecords[]>('get_daily_records', { date });
      setDailyRecords(records);
      return records;
    } catch (e) {
      console.error('Failed to load daily records:', e);
      return [];
    }
  }, []);

  const loadPeriodSummary = useCallback(async (range: DateRange) => {
    try {
      const summary = await invoke<PeriodSummary[]>('get_summary_by_range', {
        startDate: range.start,
        endDate: range.end
      });
      setPeriodSummary(summary);
      return summary;
    } catch (e) {
      console.error('Failed to load period summary:', e);
      return [];
    }
  }, []);

  const startTask = useCallback(async (taskId: string) => {
    try {
      await invoke('start_task', { taskId });
      await loadCurrentTask();
      return true;
    } catch (e) {
      console.error('Failed to start task:', e);
      return false;
    }
  }, [loadCurrentTask]);

  const stopTask = useCallback(async () => {
    try {
      await invoke('stop_task');
      setCurrentTask(null);
      return true;
    } catch (e) {
      console.error('Failed to stop task:', e);
      return false;
    }
  }, []);

  const copySummary = useCallback(async (date: string) => {
    try {
      await invoke<string>('copy_summary_to_clipboard', { date });
      return true;
    } catch (e) {
      console.error('Failed to copy summary:', e);
      return false;
    }
  }, []);

  const copyPeriodSummary = useCallback(async (range: DateRange) => {
    try {
      await invoke<string>('copy_period_summary_to_clipboard', {
        startDate: range.start,
        endDate: range.end
      });
      return true;
    } catch (e) {
      console.error('Failed to copy period summary:', e);
      return false;
    }
  }, []);

  const updateRecordDetails = useCallback(async (date: string, recordId: string, details: string | null) => {
    try {
      await invoke('update_record_details', {
        date,
        recordId,
        details: details?.trim() || null
      });
      await loadDailyRecords(date);
      return true;
    } catch (e) {
      console.error('Failed to update record details:', e);
      return false;
    }
  }, [loadDailyRecords]);

  const deleteRecord = useCallback(async (date: string, recordId: string) => {
    if (!confirm('この記録を削除しますか？')) return false;
    try {
      await invoke('delete_record', { date, recordId });
      await loadDailyRecords(date);
      return true;
    } catch (e) {
      console.error('Failed to delete record:', e);
      return false;
    }
  }, [loadDailyRecords]);

  return {
    currentTask,
    dailyRecords,
    periodSummary,
    loadCurrentTask,
    loadDailyRecords,
    loadPeriodSummary,
    startTask,
    stopTask,
    copySummary,
    copyPeriodSummary,
    updateRecordDetails,
    deleteRecord
  };
}
