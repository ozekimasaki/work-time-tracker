import { useState, useEffect, useCallback } from 'react';
import type { WorkRecord } from '../types';

export function useTimer(currentTask: WorkRecord | null) {
  const [elapsedTime, setElapsedTime] = useState(0);

  const calculateElapsedTime = useCallback(() => {
    if (!currentTask) return 0;
    const start = new Date(currentTask.start_time).getTime();
    return Math.floor((Date.now() - start) / 1000);
  }, [currentTask]);

  // タスクが変更された時に初期時間を設定
  useEffect(() => {
    setElapsedTime(calculateElapsedTime());
  }, [currentTask, calculateElapsedTime]);

  // タイマーの更新
  useEffect(() => {
    if (!currentTask) return;

    const interval = setInterval(() => {
      setElapsedTime(calculateElapsedTime());
    }, 1000);

    return () => clearInterval(interval);
  }, [currentTask, calculateElapsedTime]);

  const resetTimer = useCallback(() => {
    setElapsedTime(0);
  }, []);

  return {
    elapsedTime,
    resetTimer
  };
}
