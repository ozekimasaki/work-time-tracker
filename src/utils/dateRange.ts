/**
 * 日付計算関連のユーティリティ関数
 */

export interface DateRange {
  start: string;
  end: string;
}

/**
 * 週の範囲を取得（月曜日始まり）
 * @param weekOffset - 現在の週からのオフセット（0=今週、-1=先週、1=来週）
 */
export function getWeekRange(weekOffset: number): DateRange {
  const now = new Date();
  const dayOfWeek = now.getDay();
  const diff = now.getDate() - dayOfWeek + (dayOfWeek === 0 ? -6 : 1);
  const monday = new Date(now.setDate(diff + weekOffset * 7));
  monday.setHours(0, 0, 0, 0);
  const sunday = new Date(monday);
  sunday.setDate(monday.getDate() + 6);
  sunday.setHours(23, 59, 59, 999);
  return {
    start: monday.toISOString().split('T')[0],
    end: sunday.toISOString().split('T')[0]
  };
}

/**
 * 月の範囲を取得
 * @param monthOffset - 現在の月からのオフセット（0=今月、-1=先月、1=来月）
 */
export function getMonthRange(monthOffset: number): DateRange {
  const now = new Date();
  const year = now.getFullYear();
  const month = now.getMonth() + monthOffset;
  const firstDay = new Date(year, month, 1);
  const lastDay = new Date(year, month + 1, 0);
  return {
    start: firstDay.toISOString().split('T')[0],
    end: lastDay.toISOString().split('T')[0]
  };
}

/**
 * 今日の日付を YYYY-MM-DD 形式で取得
 */
export function getToday(): string {
  const now = new Date();
  const year = now.getFullYear();
  const month = String(now.getMonth() + 1).padStart(2, '0');
  const day = String(now.getDate()).padStart(2, '0');
  return `${year}-${month}-${day}`;
}

/**
 * 期間のラベルを取得
 */
export function getPeriodLabel(viewMode: 'weekly' | 'monthly', weekOffset: number, monthOffset: number): string {
  if (viewMode === 'weekly') {
    const range = getWeekRange(weekOffset);
    return `${range.start} ~ ${range.end}`;
  } else {
    const range = getMonthRange(monthOffset);
    const date = new Date(range.start);
    return `${date.getFullYear()}年${date.getMonth() + 1}月`;
  }
}
