/**
 * 時間フォーマット関連のユーティリティ関数
 */

/**
 * 秒数を HH:MM:SS 形式にフォーマット
 */
export function formatTime(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;
  return `${hours.toString().padStart(2, '0')}:${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
}

/**
 * 秒数を人間に優しい形式にフォーマット（X時間Y分Z秒）
 */
export function formatDuration(seconds: number): string {
  const hours = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;
  if (hours > 0) {
    return `${hours}時間${mins}分${secs}秒`;
  } else if (mins > 0) {
    return `${mins}分${secs}秒`;
  }
  return `${secs}秒`;
}

/**
 * ISO 8601日時文字列をローカル時刻にフォーマット（HH:MM:SS）
 */
export function formatDateTime(dateTimeStr: string): string {
  const date = new Date(dateTimeStr);
  return date.toLocaleTimeString('ja-JP', { hour: '2-digit', minute: '2-digit', second: '2-digit' });
}
