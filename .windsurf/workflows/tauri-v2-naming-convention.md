# Tauri v2 命名規則問題の解決方法

## 問題の概要

Tauri v2 では、フロントエンド（TypeScript/JavaScript）とバックエンド（Rust）の間で引数のキー名が一致していないとエラーが発生する。

**エラーメッセージ例:**
```
Failed to start task: invalid args `taskId` for command `start_task`: 
command start_task missing required key taskId
```

## 根本原因

- **フロントエンド**: camelCase（`taskId`, `recordId`）が一般的
- **Rust側**: snake_case（`task_id`, `record_id`）が命名規則
- Tauri v2 は自動的に変換しないため、両側のキー名が不一致になるとエラー

## 解決方法

### 1. Rust側で `#[tauri::command(rename_all = "camelCase")]` を使用

```rust
#[tauri::command(rename_all = "camelCase")]
fn start_task(
    task_id: String,  // Rust側は snake_case
    state: State<AppState>,
) -> Result<WorkRecord, String> {
    // 関数内でも snake_case を使用
    let task_name = settings
        .tasks
        .iter()
        .find(|t| t.id == task_id)  // ← task_id を使用
        .map(|t| t.name.clone())
        .unwrap_or_else(|| "不明".to_string());
    
    // ...
    *last = Some(task_id);  // ← task_id を使用
    
    Ok(new_record)
}
```

### 2. フロントエンドは camelCase を維持

```typescript
// フロントエンドは camelCase
await invoke('start_task', { taskId });
await invoke('delete_record', { date, recordId });
await invoke('update_record_details', { date, recordId, details });
```

## 適用後の状態

| 場所 | 命名規則 | 例 |
|------|----------|-----|
| Rustパラメータ | snake_case | `task_id`, `record_id` |
| Rust関数内 | snake_case | `task_id.clone()` |
| フロントエンド | camelCase | `taskId`, `recordId` |
| ビルド結果 | ✅ 警告なし | 正常動作 |

## ワークフロー（今後同様の問題が発生した場合）

### Step 1: エラーを確認
```
invalid args `xxx` for command `yyy`: command yyy missing required key xxx
```

### Step 2: 該当コマンドを特定
- `src/hooks/*.ts` で `invoke('command_name')` を検索
- `src-tauri/src/lib.rs` で `fn command_name` を検索

### Step 3: 命名規則を確認
- フロントエンドが送信しているキー名を確認（camelCaseか？）
- Rust側のパラメータ名を確認（snake_caseにする）

### Step 4: Rust側に `rename_all` アトリビュートを追加
```rust
#[tauri::command(rename_all = "camelCase")]
fn command_name(
    parameter_name: String,  // snake_case
) { }
```

### Step 5: Rust関数内の参照も snake_case に統一
関数内でパラメータを使用する箇所も snake_case にすることを忘れずに。

### Step 6: ビルド・テスト
```bash
cd src-tauri
cargo build
# 警告が出ないことを確認
```

## 注意事項

- `rename_all = "camelCase"` （正） vs `rename_all = "camel"` （誤）
- 正しい値は `"camelCase"` （CとSが大文字）
- 間違えると `expected "camelCase" or "snake_case"` エラー

## 関連ファイル

- `src/hooks/useWorkRecords.ts` - フロントエンド invoke 呼び出し
- `src/hooks/useSettings.ts` - フロントエンド invoke 呼び出し
- `src-tauri/src/lib.rs` - Rust コマンド定義
