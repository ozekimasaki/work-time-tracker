import { useState, useEffect, useCallback } from 'react';
import { listen } from '@tauri-apps/api/event';
import { useSettings } from './hooks/useSettings';
import { useWorkRecords } from './hooks/useWorkRecords';
import { useTimer } from './hooks/useTimer';
import { Header } from './components/Header';
import { TimerDisplay } from './components/TimerDisplay';
import { TaskList } from './components/TaskList';
import { SettingsPanel } from './components/SettingsPanel';
import { SummaryView } from './components/SummaryView';
import { Footer } from './components/Footer';
import { getWeekRange, getMonthRange, getToday } from './utils/dateRange';
import type { SummaryViewMode } from './types';
import './App.css';

function App() {
  const {
    settings,
    isLoading,
    saveSettings,
    addTask,
    updateTask,
    deleteTask,
    deleteAllData
  } = useSettings();

  const {
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
  } = useWorkRecords();

  const { elapsedTime, resetTimer } = useTimer(currentTask);

  const [selectedDate, setSelectedDate] = useState(getToday);
  const [showSettings, setShowSettings] = useState(false);
  const [endWorkTime, setEndWorkTime] = useState('19:00');
  const [summaryViewMode, setSummaryViewMode] = useState<SummaryViewMode>('daily');
  const [currentWeekOffset, setCurrentWeekOffset] = useState(0);
  const [currentMonthOffset, setCurrentMonthOffset] = useState(0);
  const [copied, setCopied] = useState(false);
  const [periodCopied, setPeriodCopied] = useState(false);
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false);

  useEffect(() => {
    if (settings) {
      setEndWorkTime(settings.end_work_time);
    }
  }, [settings]);

  useEffect(() => {
    loadDailyRecords(selectedDate);
  }, [selectedDate, loadDailyRecords]);

  useEffect(() => {
    const unlisten = listen('task-changed', async () => {
      console.log('[フロントエンド] task-changed イベントを受信しました');
      console.log('[フロントエンド] loadCurrentTask() を呼び出します');
      await loadCurrentTask();
      console.log('[フロントエンド] loadDailyRecords() を呼び出します');
      await loadDailyRecords(selectedDate);
      console.log('[フロントエンド] UI更新完了');
    });
    return () => {
      unlisten.then(fn => fn());
    };
  }, [loadCurrentTask, loadDailyRecords, selectedDate]);

  useEffect(() => {
    if (summaryViewMode !== 'daily') {
      const range = summaryViewMode === 'weekly'
        ? getWeekRange(currentWeekOffset)
        : getMonthRange(currentMonthOffset);
      loadPeriodSummary(range);
    }
  }, [summaryViewMode, currentWeekOffset, currentMonthOffset, loadPeriodSummary]);

  const handleStartTask = useCallback(async (taskId: string) => {
    const success = await startTask(taskId);
    if (success) {
      await loadDailyRecords(selectedDate);
    }
  }, [startTask, loadDailyRecords, selectedDate]);

  const handleStopTask = useCallback(async () => {
    const success = await stopTask();
    if (success) {
      resetTimer();
      await loadDailyRecords(selectedDate);
    }
  }, [stopTask, resetTimer, loadDailyRecords, selectedDate]);

  const handleSaveEndWorkTime = useCallback(async () => {
    if (settings) {
      const newSettings = { ...settings, end_work_time: endWorkTime };
      await saveSettings(newSettings);
    }
  }, [settings, endWorkTime, saveSettings]);

  const handleDeleteAllData = useCallback(async (): Promise<boolean> => {
    const success = await deleteAllData();
    if (success) {
      setShowDeleteConfirm(false);
      resetTimer();
      await loadDailyRecords(selectedDate);
    }
    return success;
  }, [deleteAllData, resetTimer, loadDailyRecords, selectedDate]);

  const handleCopySummary = useCallback(async () => {
    const success = await copySummary(selectedDate);
    if (success) {
      setCopied(true);
      setTimeout(() => setCopied(false), 2000);
    }
  }, [copySummary, selectedDate]);

  const handleCopyPeriodSummary = useCallback(async () => {
    const range = summaryViewMode === 'weekly'
      ? getWeekRange(currentWeekOffset)
      : getMonthRange(currentMonthOffset);
    const success = await copyPeriodSummary(range);
    if (success) {
      setPeriodCopied(true);
      setTimeout(() => setPeriodCopied(false), 2000);
    }
  }, [copyPeriodSummary, summaryViewMode, currentWeekOffset, currentMonthOffset]);

  const handleDateChange = useCallback((date: string) => {
    setSelectedDate(date);
  }, []);

  const handleViewModeChange = useCallback((mode: SummaryViewMode) => {
    setSummaryViewMode(mode);
  }, []);

  if (isLoading || !settings) {
    return <div className="loading">読み込み中...</div>;
  }

  return (
    <div className="app">
      <Header onToggleSettings={() => setShowSettings(!showSettings)} />

      <TimerDisplay
        elapsedTime={elapsedTime}
        currentTask={currentTask}
        onStopTask={handleStopTask}
      />

      {!showSettings && (
        <TaskList
          tasks={settings.tasks}
          currentTask={currentTask}
          onStartTask={handleStartTask}
        />
      )}

      {showSettings && (
        <SettingsPanel
          settings={settings}
          endWorkTime={endWorkTime}
          showDeleteConfirm={showDeleteConfirm}
          onEndWorkTimeChange={setEndWorkTime}
          onSaveEndWorkTime={handleSaveEndWorkTime}
          onAddTask={addTask}
          onUpdateTask={updateTask}
          onDeleteTask={deleteTask}
          onShowDeleteConfirm={setShowDeleteConfirm}
          onDeleteAllData={handleDeleteAllData}
        />
      )}

      <SummaryView
        viewMode={summaryViewMode}
        dailyRecords={dailyRecords}
        periodSummary={periodSummary}
        selectedDate={selectedDate}
        copied={copied}
        periodCopied={periodCopied}
        currentWeekOffset={currentWeekOffset}
        currentMonthOffset={currentMonthOffset}
        onViewModeChange={handleViewModeChange}
        onDateChange={handleDateChange}
        onCopySummary={handleCopySummary}
        onCopyPeriodSummary={handleCopyPeriodSummary}
        onWeekOffsetChange={setCurrentWeekOffset}
        onMonthOffsetChange={setCurrentMonthOffset}
        onUpdateRecordDetails={updateRecordDetails}
        onDeleteRecord={deleteRecord}
      />

      <Footer />
    </div>
  );
}

export default App;
