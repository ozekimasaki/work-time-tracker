import { useState } from 'react';
import { ChevronDown, Copy, Check, Edit2, Trash2, FileText, X, ChevronLeft, ChevronRight } from 'lucide-react';
import { formatDateTime, formatDuration } from '../utils/timeFormat';
import { getPeriodLabel } from '../utils/dateRange';
import type { WorkRecord, TaskWithRecords, PeriodSummary, SummaryViewMode } from '../types';

interface SummaryViewProps {
  viewMode: SummaryViewMode;
  dailyRecords: TaskWithRecords[];
  periodSummary: PeriodSummary[];
  selectedDate: string;
  copied: boolean;
  periodCopied: boolean;
  currentWeekOffset: number;
  currentMonthOffset: number;
  onViewModeChange: (mode: SummaryViewMode) => void;
  onDateChange: (date: string) => void;
  onCopySummary: () => void;
  onCopyPeriodSummary: () => void;
  onWeekOffsetChange: (offset: number) => void;
  onMonthOffsetChange: (offset: number) => void;
  onUpdateRecordDetails: (date: string, recordId: string, details: string | null) => Promise<boolean>;
  onDeleteRecord: (date: string, recordId: string) => Promise<boolean>;
}

export function SummaryView({
  viewMode,
  dailyRecords,
  periodSummary,
  selectedDate,
  copied,
  periodCopied,
  currentWeekOffset,
  currentMonthOffset,
  onViewModeChange,
  onDateChange,
  onCopySummary,
  onCopyPeriodSummary,
  onWeekOffsetChange,
  onMonthOffsetChange,
  onUpdateRecordDetails,
  onDeleteRecord
}: SummaryViewProps) {
  const [expandedTasks, setExpandedTasks] = useState<Set<string>>(new Set());
  const [editingRecord, setEditingRecord] = useState<{taskName: string, recordId: string} | null>(null);
  const [recordDetails, setRecordDetails] = useState('');

  const toggleTaskExpansion = (taskName: string) => {
    setExpandedTasks(prev => {
      const newSet = new Set(prev);
      if (newSet.has(taskName)) {
        newSet.delete(taskName);
      } else {
        newSet.add(taskName);
      }
      return newSet;
    });
  };

  const startEditingRecord = (taskName: string, record: WorkRecord) => {
    setEditingRecord({ taskName, recordId: record.id });
    setRecordDetails(record.details || '');
  };

  const saveRecordDetails = async (date: string, recordId: string) => {
    const success = await onUpdateRecordDetails(date, recordId, recordDetails.trim() || null);
    if (success) {
      setEditingRecord(null);
      setRecordDetails('');
    }
  };

  const cancelEditingRecord = () => {
    setEditingRecord(null);
    setRecordDetails('');
  };

  const handleDeleteRecord = async (date: string, recordId: string) => {
    const success = await onDeleteRecord(date, recordId);
    if (success && editingRecord?.recordId === recordId) {
      setEditingRecord(null);
      setRecordDetails('');
    }
  };

  const handleDateChange = (e: React.ChangeEvent<HTMLInputElement>) => {
    onDateChange(e.target.value);
    setExpandedTasks(new Set());
    setEditingRecord(null);
    setRecordDetails('');
  };

  return (
    <section className="daily-summary">
      <div className="summary-header">
        <h2>作業記録</h2>
        <div className="view-tabs">
          <button className={viewMode === 'daily' ? 'active' : ''} onClick={() => onViewModeChange('daily')}>日次</button>
          <button className={viewMode === 'weekly' ? 'active' : ''} onClick={() => onViewModeChange('weekly')}>週次</button>
          <button className={viewMode === 'monthly' ? 'active' : ''} onClick={() => onViewModeChange('monthly')}>月次</button>
        </div>
      </div>

      {viewMode === 'daily' && (
        <>
          <div className="summary-header-row">
            <div className="daily-date-row">
              <span className="period-label">日付</span>
              <input
                className="date-picker"
                type="date"
                value={selectedDate}
                onChange={handleDateChange}
              />
            </div>
            <button className={`copy-btn ${copied ? 'copied' : ''}`} onClick={onCopySummary}>
              {copied ? <><Check size={16} /> コピー完了</> : <><Copy size={16} /> コピー</>}
            </button>
          </div>
          <div className="summary-list">
            {dailyRecords.length === 0 ? (
              <div className="no-records">記録はありません</div>
            ) : (
              dailyRecords.map((taskRecord) => (
                <div key={taskRecord.task_name} className="task-record">
                  <div className="summary-item clickable" onClick={() => toggleTaskExpansion(taskRecord.task_name)}>
                    <div className="task-info">
                      <ChevronDown size={16} className={`chevron ${expandedTasks.has(taskRecord.task_name) ? 'expanded' : ''}`} />
                      <span className="name">{taskRecord.task_name}</span>
                      <span className="record-count">({taskRecord.records.length}件)</span>
                    </div>
                    <span className="time">{formatDuration(taskRecord.total_seconds)}</span>
                  </div>
                  {expandedTasks.has(taskRecord.task_name) && (
                    <div className="record-details">
                      {taskRecord.records.map((record) => (
                        <div key={record.id} className="record-item">
                          {editingRecord?.taskName === taskRecord.task_name && editingRecord?.recordId === record.id ? (
                            <div className="record-edit-form">
                              <div className="record-time-row">
                                <span className="record-time">{formatDateTime(record.start_time)} - {record.end_time ? formatDateTime(record.end_time) : '進行中'}</span>
                                <span className="record-duration">{record.duration_seconds ? formatDuration(record.duration_seconds) : formatDuration(Math.floor((Date.now() - new Date(record.start_time).getTime()) / 1000))}</span>
                              </div>
                              <textarea 
                                className="record-details-input" 
                                placeholder="作業内容の詳細を入力..." 
                                value={recordDetails} 
                                onChange={(e) => setRecordDetails(e.target.value)} 
                                rows={2} 
                              />
                              <div className="record-edit-actions">
                                <button onClick={() => saveRecordDetails(selectedDate, record.id)}><Check size={14} /> 保存</button>
                                <button onClick={cancelEditingRecord}><X size={14} /> キャンセル</button>
                              </div>
                            </div>
                          ) : (
                            <>
                              <div className="record-info-row">
                                <span className="record-time">{formatDateTime(record.start_time)} - {record.end_time ? formatDateTime(record.end_time) : '進行中'}</span>
                                <div className="record-actions">
                                  <span className="record-duration">{record.duration_seconds ? formatDuration(record.duration_seconds) : formatDuration(Math.floor((Date.now() - new Date(record.start_time).getTime()) / 1000))}</span>
                                  <button className="edit-record-btn" onClick={() => startEditingRecord(taskRecord.task_name, record)} title="詳細を編集"><Edit2 size={14} /></button>
                                  {record.end_time && (
                                    <button className="delete-record-btn" onClick={() => handleDeleteRecord(selectedDate, record.id)} title="記録を削除"><Trash2 size={14} /></button>
                                  )}
                                </div>
                              </div>
                              {record.details && <div className="record-details-text"><FileText size={12} /> {record.details}</div>}
                            </>
                          )}
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              ))
            )}
          </div>
        </>
      )}

      {(viewMode === 'weekly' || viewMode === 'monthly') && (
        <>
          <div className="summary-header-row">
            <div className="period-nav">
              <button className="nav-btn" onClick={() => viewMode === 'weekly' ? onWeekOffsetChange(currentWeekOffset - 1) : onMonthOffsetChange(currentMonthOffset - 1)}>
                <ChevronLeft size={16} />
              </button>
              <span className="period-label">{getPeriodLabel(viewMode, currentWeekOffset, currentMonthOffset)}</span>
              <button className="nav-btn" onClick={() => viewMode === 'weekly' ? onWeekOffsetChange(currentWeekOffset + 1) : onMonthOffsetChange(currentMonthOffset + 1)}>
                <ChevronRight size={16} />
              </button>
            </div>
            <button className={`copy-btn ${periodCopied ? 'copied' : ''}`} onClick={onCopyPeriodSummary}>
              {periodCopied ? <><Check size={16} /> コピー完了</> : <><Copy size={16} /> コピー</>}
            </button>
          </div>
          <div className="summary-list">
            {periodSummary.length === 0 ? (
              <div className="no-records">{viewMode === 'weekly' ? 'この週の記録はありません' : 'この月の記録はありません'}</div>
            ) : (
              periodSummary.map((taskSummary) => (
                <div key={taskSummary.task_name} className="task-record">
                  <div className="summary-item clickable" onClick={() => toggleTaskExpansion(taskSummary.task_name)}>
                    <div className="task-info">
                      <ChevronDown size={16} className={`chevron ${expandedTasks.has(taskSummary.task_name) ? 'expanded' : ''}`} />
                      <span className="name">{taskSummary.task_name}</span>
                    </div>
                    <span className="time">{formatDuration(taskSummary.total_seconds)}</span>
                  </div>
                  {expandedTasks.has(taskSummary.task_name) && (
                    <div className="record-details">
                      {taskSummary.daily_breakdown.map((daily) => (
                        <div key={daily.date} className="record-item period-daily-item">
                          <span className="daily-date">{daily.date}</span>
                          <span className="daily-time">{formatDuration(daily.total_seconds)}</span>
                        </div>
                      ))}
                    </div>
                  )}
                </div>
              ))
            )}
          </div>
        </>
      )}
    </section>
  );
}
