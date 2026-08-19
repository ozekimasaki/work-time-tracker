# AGENTS.md

このリポジトリでコーディングエージェントが作業するためのガイドです。人間向けの概要は [`README.md`](./README.md)、詳細仕様は [`SPECIFICATION.md`](./SPECIFICATION.md) を参照してください。

## プロジェクト概要

作業時間を記録するデスクトップアプリです。Tauri v2 をシェルに、フロントエンドは React 19 + TypeScript（Vite 7）、バックエンドは Rust（edition 2021）で実装されています。

## プロジェクト構成 / エントリポイント

- `index.html`: Vite のエントリー HTML。`src/main.tsx` を読み込みます。
- `src/main.tsx`: React のエントリポイント。`App` を `#root` にマウント。
- `src/App.tsx`: メインコンポーネント。状態管理と各コンポーネントの結合を担当。
- `src/components/`: UI コンポーネント（`Header` / `TimerDisplay` / `TaskList` / `SettingsPanel` / `SummaryView` / `Footer`）。
- `src/hooks/`: カスタムフック（`useSettings` / `useWorkRecords` / `useTimer`）。Tauri コマンド呼び出しはここに集約されています。
- `src/utils/`: `timeFormat.ts`（時間フォーマット）、`dateRange.ts`（日付範囲計算）。
- `src/types.ts`: フロントエンドの型定義。
- `src/App.css`: スタイル（プレーン CSS。Tailwind 等の CSS フレームワークは未使用）。
- `src-tauri/src/lib.rs`: バックエンドの中心。全 Tauri コマンド、システムトレイ、グローバルショートカット、終業リマインダーを実装。
- `src-tauri/src/main.rs`: バイナリのエントリポイント（`work_time_tracker_lib::run()` を呼ぶだけ）。
- `src-tauri/tauri.conf.json`: Tauri 設定（ウィンドウ、バンドルターゲット、`beforeDevCommand`/`beforeBuildCommand` 等）。
- `src-tauri/capabilities/default.json`: 権限（capability）定義。新しいプラグイン API を使う場合はここに権限追加が必要です。
- `src-tauri/Cargo.toml`: Rust 依存関係。

## セットアップ

```bash
npm install
```

Rust 側の依存は Cargo が自動取得します。デスクトップアプリのビルド・実行には Tauri v2 のプラットフォーム別前提条件（https://v2.tauri.app/start/prerequisites/ ）が必要です。ネイティブ依存の一部が新しめの Cargo（`edition2024` 対応）を要求するため、Rust は最新の安定版を使用してください（CI は `dtolnay/rust-toolchain@stable`）。

## ビルド / テスト / lint / typecheck コマンド

`package.json` のスクリプト:

- `npm run dev`: Vite 開発サーバー（ポート 1420、固定）。
- `npm run build`: `tsc && vite build`。TypeScript の型チェックを兼ねる本番ビルド。
- `npm run preview`: ビルド済みフロントエンドのプレビュー。
- `npm run tauri`: Tauri CLI。
- `npm run tauri:dev`: デスクトップアプリを開発モードで起動。
- `npm run tauri:build`: デスクトップアプリのリリースビルド。

型チェック / lint / フォーマット（専用スクリプトは未定義のため直接実行）:

- 型チェック（フロントエンド）: `npx tsc --noEmit`
- Rust フォーマット: `src-tauri` 内で `cargo fmt --all`（確認のみは `cargo fmt --all -- --check`）
- Rust lint: `src-tauri` 内で `cargo clippy`
- Rust 型 / ビルドチェック: `src-tauri` 内で `cargo check`

**テスト**: 自動テストのフレームワーク・テストコードはこのリポジトリに存在しません。テストを新設する場合は、まず方針を確認してください。

変更後は、少なくとも影響範囲に対して型チェック（`npx tsc --noEmit`）を実行し、エラーがないことを確認してください。Rust を変更した場合は `cargo check` / `cargo clippy` を実行してください。

## コーディング規約

- **言語**: フロントエンドは TypeScript（`strict: true`）。`tsconfig.json` で `noUnusedLocals` / `noUnusedParameters` / `noFallthroughCasesInSwitch` が有効なため、未使用変数・引数を残さないこと。
- **React**: 関数コンポーネント + フック。副作用は `useEffect`、関数のメモ化は `useCallback` を使用（既存コードの流儀に合わせる）。Tauri の `invoke` 呼び出しはコンポーネントに散らさず、`src/hooks/` に集約する。
- **Rust ↔ フロントエンドの命名規則**: フロントエンドは camelCase、Rust のコマンド引数は snake_case です。引数を持つコマンドでは Rust 側に `#[tauri::command(rename_all = "camelCase")]` を付け、フロントエンドからは camelCase のキーで渡します（例: `invoke('start_task', { taskId })` ↔ `fn start_task(task_id: String, ...)`）。詳細は [`.windsurf/workflows/tauri-v2-naming-convention.md`](./.windsurf/workflows/tauri-v2-naming-convention.md) を参照。
- **UI 文言**: アプリの UI テキスト・通知・ログは日本語です。既存に合わせてください。
- **スタイル**: `src/App.css` にプレーン CSS で記述。CSS フレームワークは導入しないこと。
- **アイコン**: `lucide-react` を使用。
- このリポジトリには `.windsurf/rules/main.md` にコーディング支援ルールがあります。作業前に確認してください。

## 新しいコマンド / プラグインを追加する場合

1. `src-tauri/src/lib.rs` に `#[tauri::command]`（引数があれば `rename_all = "camelCase"`）で関数を実装する。
2. `run()` 内の `tauri::generate_handler![...]` に関数を登録する。
3. 新しいプラグイン API を使う場合は `src-tauri/Cargo.toml` に依存を追加し、`run()` で `.plugin(...)` を登録し、`src-tauri/capabilities/default.json` に必要な権限を追記する。
4. フロントエンド側は `src/hooks/` に `invoke('command_name', {...})` を追加し、必要なら `src/types.ts` に型を追加する。Rust と TypeScript の型・キー名の対応を必ず合わせること。

## 注意点

- **データ保存**: 設定と日別記録は Tauri の app data ディレクトリ配下 `com.masam.work-time-tracker/work-time-tracker/` に JSON（`settings.json` と `YYYY-MM-DD.json`）で保存されます。`delete_all_data` は `settings.json` 以外の `.json` を削除します。
- **状態管理**: バックエンドの状態は `AppState`（`Mutex` で保護）で保持します。ロックの取り扱いに注意してください。
- **イベント連携**: トレイ／ショートカット経由で状態が変わると Rust 側が `task-changed` イベントを emit し、フロントエンドはこれを購読して UI を更新します。作業状態を変える処理を追加したら、必要に応じてこのイベントを emit してください。
- **プラットフォーム**: 配布ターゲットは Windows（`nsis`）と macOS（`app` / `dmg`）です。トレイのショートカット表記は macOS で `Cmd`、それ以外で `Ctrl` に切り替わります。Linux ではビルドに追加の system 依存が必要です。
- **ポート**: Vite は 1420 番ポート固定（`strictPort: true`）で、Tauri がこれを前提にしています。変更しないでください。
- **変更範囲**: 生成物（`dist/`、`src-tauri/target/`）はコミットしないこと（`.gitignore` 済み）。`src-tauri/Cargo.lock` は追跡対象なので、依存を変えたときのみ更新をコミットしてください。
