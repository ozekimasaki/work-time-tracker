# 作業時間記録アプリ - 仕様書・実装まとめ

## 概要

Windows向けの作業時間記録デスクトップアプリケーション。最小操作で複数の作業を切り替えながら時間を記録し、日別のサマリーをテキスト形式で出力できる。

---

## 技術スタック

| 項目 | 技術 |
|------|------|
| フレームワーク | Tauri v2 |
| フロントエンド | React + TypeScript |
| バックエンド | Rust |
| UIスタイル | TailwindCSS |
| アイコン | lucide-react |

---

## 機能仕様

### 1. 基本機能

#### 作業記録
- **開始**: 作業ボタンクリック or ショートカットキー
- **停止**: 停止ボタン or ショートカットキー
- **自動保存**: 停止時に自動的に日別JSONファイルに保存
- **秒単位記録**: 1秒以上の作業を記録

#### 作業管理（CRUD）
| 操作 | 方法 |
|------|------|
| 作成 | 設定画面で作業名と色を指定して追加 |
| 読み込み | アプリ起動時に自動読み込み |
| 更新 | 設定画面で編集アイコンをクリック |
| 削除 | 設定画面で削除アイコンをクリック |

#### ショートカットキー
| キー | 機能 |
|------|------|
| `Ctrl+Shift+S` | 現在の作業を停止 |
| `Ctrl+Shift+R` | 直前の作業を再開 |
| `Ctrl+Shift+1~9` | プリセット作業1~9に切り替え |

### 2. 終業リマインダー
- 設定可能な時刻（デフォルト: 19:00）
- 時間になると通知 + アプリウィンドウを前面表示

### 3. データ出力
- **コピー形式**: 「作業名: XX時間XX分XX秒」
- **詳細出力**: 開始時間・終了時間・作業時間を含む詳細テキスト

---

## データ構造

### Rust（バックエンド）

```rust
// 作業定義
pub struct Task {
    pub id: String,
    pub name: String,
    pub color: Option<String>,
    pub shortcut_index: Option<u8>, // 1-9 for Ctrl+Shift+1~9
}

// 作業記録
pub struct WorkRecord {
    pub id: String,
    pub task_id: String,
    pub task_name: String,
    pub start_time: DateTime<Local>,
    pub end_time: Option<DateTime<Local>>,
    pub duration_seconds: Option<i64>,
}

// 日別ログ
pub struct DailyLog {
    pub date: String, // YYYY-MM-DD
    pub records: Vec<WorkRecord>,
}

// 設定
pub struct Settings {
    pub tasks: Vec<Task>,
    pub end_work_time: String, // HH:MM
    pub reminder_enabled: bool,
}
```

### TypeScript（フロントエンド）

```typescript
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
  duration_seconds?: number;
}

export interface Settings {
  tasks: Task[];
  end_work_time: string;
  reminder_enabled: boolean;
}
```

---

## ファイル構成

```
work-time-tracker/
├── src/                          # Reactフロントエンド
│   ├── App.tsx                  # メインUIコンポーネント
│   ├── App.css                  # スタイル定義
│   ├── types.ts                 # TypeScript型定義
│   ├── main.tsx                 # エントリーポイント
│   └── vite-env.d.ts            # Vite型定義
├── src-tauri/                    # Rustバックエンド
│   ├── src/
│   │   └── lib.rs               # メイン実装
│   ├── Cargo.toml               # Rust依存関係
│   ├── tauri.conf.json          # Tauri設定
│   └── icons/                   # アプリアイコン
├── package.json                 # npm設定
├── tsconfig.json                # TypeScript設定
└── vite.config.ts               # Vite設定
```

---

## データ保存場所

Windows: `%APPDATA%/work-time-tracker/`

### ファイル構成
```
%APPDATA%/work-time-tracker/
├── settings.json          # 作業プリセット・設定
├── 2025-01-31.json        # 日別記録（例）
└── 2025-02-01.json        # 日別記録（例）
```

---

## 実装詳細

### バックエンド（Rust）

#### 主要コマンド

| コマンド | 説明 |
|----------|------|
| `get_settings` | 設定を取得 |
| `save_settings_command` | 設定を保存 |
| `start_task` | 作業を開始（前の作業があれば停止・保存） |
| `stop_task` | 現在の作業を停止・保存 |
| `get_current_task` | 現在進行中の作業を取得 |
| `get_daily_summary` | 指定日のサマリーを取得 |
| `copy_summary_to_clipboard` | クリップボードに詳細サマリーをコピー |
| `add_task` | 新規作業を追加 |
| `update_task` | 作業を更新 |
| `delete_task` | 作業を削除 |

#### システムトレイ
- 左クリック: ウィンドウ表示
- 右クリック: メニュー表示（表示/停止/再開/終了）

#### グローバルショートカット
- Tauriプラグイン `tauri-plugin-global-shortcut` を使用
- `on_shortcut` でハンドラーを登録

#### 通知
- Tauriプラグイン `tauri-plugin-notification` を使用
- 終業時刻に通知を表示

### フロントエンド（React）

#### 状態管理
- `useState` でローカル状態を管理
- `useEffect` で定期的にデータを更新（5秒間隔）
- `useCallback` で関数をメモ化

#### タイマー表示
```typescript
const formatTime = (seconds: number) => {
  const hours = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;
  return `${hours.toString().padStart(2, '0')}:${mins.toString().padStart(2, '0')}:${secs.toString().padStart(2, '0')}`;
};
```

#### 作業時間フォーマット
```typescript
const formatDuration = (seconds: number) => {
  const hours = Math.floor(seconds / 3600);
  const mins = Math.floor((seconds % 3600) / 60);
  const secs = seconds % 60;
  if (hours > 0) return `${hours}時間${mins}分${secs}秒`;
  if (mins > 0) return `${mins}分${secs}秒`;
  return `${secs}秒`;
};
```

---

## UI構成

### メイン画面
```
┌─────────────────────────┐
│ 作業時間記録      [⚙️]  │  ← ヘッダー（設定ボタン）
├─────────────────────────┤
│ [⏱] 00:00:00          │  ← タイマー表示
│ 現在の作業: [停止]      │  ← 進行中作業
├─────────────────────────┤
│ 作業一覧                │
│ ┌─────────────────────┐ │
│ │ Ctrl+Shift+1  会議  │ │  ← 作業ボタン
│ │ Ctrl+Shift+2  コーディング │ │
│ │ ...                 │ │
│ └─────────────────────┘ │
├─────────────────────────┤
│ 本日の記録        [📋]  │  ← コピーボタン
│ ┌─────────────────────┐ │
│ │ 会議: 1時間30分15秒 │ │
│ │ コーディング: 45分20秒 │ │
│ └─────────────────────┘ │
├─────────────────────────┤
│ [Ctrl+Shift+S] 停止    │  ← ショートカットヘルプ
│ [Ctrl+Shift+R] 再開    │
│ [Ctrl+Shift+1~9] 切替  │
└─────────────────────────┘
```

### 設定画面
```
┌─────────────────────────┐
│ 設定              [✕]  │
├─────────────────────────┤
│ 🔔 終業時刻リマインダー │
│ [19:00] [保存]          │
├─────────────────────────┤
│ 作業一覧                │
│ ┌─────────────────────┐ │
│ │ 会議          [🎨][✏][🗑]│ │
│ │ コーディング  [🎨][✏][🗑]│ │
│ └─────────────────────┘ │
├─────────────────────────┤
│ 新規作業追加            │
│ [作業名入力]            │
│ [🎨🎨🎨...] [追加]     │
└─────────────────────────┘
```

---

## ビルド・配布

### 開発モードで起動
```powershell
cd c:\Users\masam\Documents\work-time-tracker
$env:PATH += ";C:\Users\masam\.cargo\bin"
npm run tauri dev
```

### リリースビルド（インストーラー作成）
```powershell
cd c:\Users\masam\Documents\work-time-tracker
$env:PATH += ";C:\Users\masam\.cargo\bin"
npm run tauri build
```

### 出力先
- Windowsインストーラー: `src-tauri/target/release/bundle/msi/*.msi`

---

## 今後の改善案

1. **週次/月次サマリー**: 長期的な統計機能
2. **カテゴリ分類**: 作業のカテゴリ別集計
3. **休憩時間記録**: 自動休憩検出
4. **アイドル検出**: 離席時の自動停止
5. **データエクスポート**: CSV/Excel出力
6. **バックアップ**: クラウド同期機能

---

## 作成日

2025年1月31日
