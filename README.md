# 作業時間記録アプリ（work-time-tracker）

複数の作業を切り替えながら作業時間を秒単位で記録し、日別・週別・月別のサマリーをテキスト形式で出力できるデスクトップアプリケーションです。Tauri v2（Rust）+ React + TypeScript で実装されています。

## 概要

- 作業ボタンまたはグローバルショートカットで作業の開始・停止・切り替えができます。
- 停止時に日別の JSON ファイルへ自動保存します。
- 記録は作業（タスク）ごとに集計され、日別／週別／月別で確認できます。
- サマリーはクリップボードへテキストとしてコピーできます。
- 設定した終業時刻に通知を表示し、ウィンドウを前面に表示します。
- システムトレイに常駐し、最小操作で利用できます。

ウィンドウは幅 500 × 高さ 800 の固定サイズ（リサイズ不可）で、ウィンドウを閉じるとトレイに格納されます。

## 主な機能

- **作業記録**: 開始・停止・切り替え。秒単位で記録し、停止時に自動保存。
- **作業（タスク）管理**: 作業名・色を指定した追加・編集・削除。1〜9 番のショートカット枠を自動割り当て。
- **グローバルショートカット**:
  - `Ctrl+Shift+S`: 現在の作業を停止
  - `Ctrl+Shift+R`: 直前の作業を再開
  - `Ctrl+Shift+1`〜`Ctrl+Shift+9`: 対応するプリセット作業に切り替え
- **サマリー表示**: 日別（記録詳細つき）／週別／月別。作業ごとの合計と日別内訳を表示。
- **記録の編集**: 各記録にメモ（詳細）を追加・編集、記録の個別削除。
- **クリップボードコピー**: 日別サマリー／期間サマリーをテキスト形式でコピー。
- **終業リマインダー**: 設定時刻（既定 19:00）に通知を表示してウィンドウを前面化。
- **全データ削除**: 設定画面から日別記録をまとめて削除（`settings.json` は保持）。
- **システムトレイ**: 左クリックで表示、右クリックメニューで表示／停止／再開／終了。

## 技術スタック

| 項目 | 内容 |
|------|------|
| フレームワーク | Tauri v2 |
| フロントエンド | React 19 + TypeScript |
| バックエンド | Rust（edition 2021） |
| ビルドツール | Vite 7 |
| アイコン | lucide-react |
| スタイル | プレーン CSS（`src/App.css`） |

主な Tauri プラグイン: `global-shortcut`、`clipboard-manager`、`notification`、`opener`。

## 要件

- Node.js（Vite 7 は Node.js 20.19+ または 22.12+ を推奨）
- npm
- Rust ツールチェイン（安定版。ネイティブ依存に新しめの Cargo が必要です。CI は `dtolnay/rust-toolchain@stable` を使用）
- Tauri v2 のプラットフォーム別前提条件（詳細は https://v2.tauri.app/start/prerequisites/ を参照）

## インストール

```bash
git clone https://github.com/ozekimasaki/work-time-tracker.git
cd work-time-tracker
npm install
```

## 使い方

デスクトップアプリとして開発モードで起動します（Rust のバックエンドを含めてビルド・起動されます）。

```bash
npm run tauri:dev
```

インストーラーを含むリリースビルドを作成します。

```bash
npm run tauri:build
```

`npm run tauri:build` の成果物は `src-tauri/target/release/bundle/` 以下に生成されます（設定上のターゲットは `nsis`・`app`・`dmg`）。

> フロントエンド単体の `npm run dev` / `npm run preview` は Vite の開発サーバー（`http://localhost:1420`）を起動しますが、`@tauri-apps/api` を介した機能はデスクトップアプリ（`tauri:dev`）上でのみ動作します。

## 開発コマンド

`package.json` に定義されているスクリプト:

| コマンド | 説明 |
|----------|------|
| `npm run dev` | Vite 開発サーバーを起動（ポート 1420、固定） |
| `npm run build` | 型チェック（`tsc`）後に Vite で本番ビルド |
| `npm run preview` | ビルド済みフロントエンドをプレビュー |
| `npm run tauri` | Tauri CLI を実行 |
| `npm run tauri:dev` | デスクトップアプリを開発モードで起動 |
| `npm run tauri:build` | デスクトップアプリをリリースビルド |

補助的なチェック（スクリプト未定義のため直接実行）:

```bash
# TypeScript の型チェックのみ
npx tsc --noEmit

# Rust 側（src-tauri ディレクトリ内で実行）
cargo fmt --all
cargo clippy
cargo check
```

## プロジェクト構成

```
work-time-tracker/
├── index.html                 # Vite のエントリー HTML
├── package.json               # npm スクリプト・依存関係
├── tsconfig.json              # TypeScript 設定
├── vite.config.ts             # Vite 設定（ポート 1420 等）
├── src/                       # React フロントエンド
│   ├── main.tsx               # エントリーポイント
│   ├── App.tsx                # メインコンポーネント
│   ├── App.css                # スタイル
│   ├── types.ts               # 型定義
│   ├── components/            # UI コンポーネント
│   │   ├── Header.tsx
│   │   ├── TimerDisplay.tsx
│   │   ├── TaskList.tsx
│   │   ├── SettingsPanel.tsx
│   │   ├── SummaryView.tsx
│   │   └── Footer.tsx
│   ├── hooks/                 # カスタムフック
│   │   ├── useSettings.ts
│   │   ├── useWorkRecords.ts
│   │   └── useTimer.ts
│   └── utils/                 # ユーティリティ
│       ├── timeFormat.ts
│       └── dateRange.ts
├── src-tauri/                 # Rust バックエンド
│   ├── src/
│   │   ├── lib.rs             # コマンド・トレイ・ショートカット・通知の実装
│   │   └── main.rs            # バイナリのエントリーポイント
│   ├── capabilities/default.json  # 権限（capability）定義
│   ├── tauri.conf.json        # Tauri 設定
│   ├── Cargo.toml             # Rust 依存関係
│   └── icons/                 # アプリアイコン
├── SPECIFICATION.md           # 詳細仕様・実装メモ
└── .github/workflows/release.yml  # タグ push 時のリリースビルド
```

より詳細な仕様・データ構造・コマンド一覧は [`SPECIFICATION.md`](./SPECIFICATION.md) を参照してください。

## データ保存場所

設定と日別記録は Tauri の app data ディレクトリ配下の `com.masam.work-time-tracker/work-time-tracker/` に保存されます。

```
<app data dir>/com.masam.work-time-tracker/work-time-tracker/
├── settings.json          # 作業プリセット・設定
├── 2025-01-31.json        # 日別記録（例）
└── 2025-02-01.json        # 日別記録（例）
```

- Windows: `%APPDATA%\com.masam.work-time-tracker\work-time-tracker\`
- macOS: `~/Library/Application Support/com.masam.work-time-tracker/work-time-tracker/`

## 推奨 IDE 設定

- [VS Code](https://code.visualstudio.com/) + [Tauri](https://marketplace.visualstudio.com/items?itemName=tauri-apps.tauri-vscode) + [rust-analyzer](https://marketplace.visualstudio.com/items?itemName=rust-lang.rust-analyzer)

## リリース

`v*` 形式のタグを push すると、GitHub Actions（[`.github/workflows/release.yml`](./.github/workflows/release.yml)）が Windows・macOS 向けにビルドし、GitHub Release を作成します。手動実行（`workflow_dispatch`）にも対応しています。

## ライセンス

このリポジトリには LICENSE ファイルがなく、`package.json` は `private` 指定です。ライセンスは明示されていません。
