# 変更履歴（Changelog）

このプロジェクトで実施した注目すべき変更をすべて記録します。

## [0.1.0] - 2026-06-08

### Phase 6B: Electron GUI 移行 - 完了 ✅

#### Phase 6B-0: 共有型の抽出
- egui crate から `PipelineState`, `Node`, `Edge` を `ajisai-core` へ移動
- Rust edition を 2024 に更新
- egui GUI との後方互換性を維持

#### Phase 6B-1: JSON-RPC サーバー
- `crates/server` に stdio JSON-RPC プロトコルを実装
- 6 つのコアメソッドを実装: ping, get_transforms, run_pipeline, validate_pipeline, load_pipeline, save_pipeline
- stdout 書き込みタスクを単一化してフレーム衝突を防止
- Serde による完全なシリアライゼーション対応

#### Phase 6B-2: Electron スキャフォールド
- `electron/` ディレクトリを electron-vite で初期化
- メインプロセスに Rust バイナリのサイドカー管理を実装
- contextBridge API (`window.ajisai`) を追加
- Zustand ストアで状態管理
- パイプラインモデルの TypeScript 型定義

#### Phase 6B-3: キャンバス + UI コンポーネント
- @xyflow/react v12 を使用した DAG ビジュアルエディタ
- トランスフォームパレットサイドバー（カテゴリ分類）
- JSON コンフィグエディタ付きプロパティパネル
- 自動スクロール対応ログパネル（最大 500 行履歴）
- リアルタイム実行インジケータ付きステータスバー
- VS Code ダークテーマ対応のレスポンシブレイアウト

#### Phase 6B-4: ファイル I/O + キーボード操作
- File / Pipeline メニューバー実装
- キーボードショートカット: Ctrl+N, Ctrl+O, Ctrl+S, Ctrl+R
- ストリーミング進捗表示対応パイプライン実行
- ノード状態 + サーバーメッセージのリアルタイムログ表示

### Phase 6C: GUI ポーランド・最適化 - 完了 ✅

#### Phase 6C-1: トランスフォーム別フォーム定義
- 50+ トランスフォームの型付きフィールド定義
- フォーム型の自動検出（文字列, 数値, ブール値, セレクト, テキストエリア, JSON）
- リアルタイム JSON 検証
- 型別 UI 生成用 FormRenderer
- 不明なトランスフォームは JSON テキストエリアにフォールバック

#### Phase 6C-2: Undo/Redo 統合
- Zustand ストアに 50 スナップショットの Undo/Redo スタック実装（egui 互換）
- Ctrl+Z / Ctrl+Y / Ctrl+Shift+Z キーバインディング
- 新規変更時に自動的に Redo スタックをクリア
- `canUndo()` / `canRedo()` 状態クエリ対応

#### Phase 6C-3: ウィンドウ状態の永続化
- パイプライン変更時に localStorage に自動保存
- アプリ起動時にパイプライン自動復元
- 自動保存で誤った作業喪失を防止

#### Phase 6C-4: パフォーマンス最適化
- Canvas コンポーネントに React.memo() を適用
- useMemo() フックで不要なレンダリングを防止
- バンドルサイズ: JS 606.94 kB + CSS 32.82 kB
- スナップショットベース Undo による効率的な状態更新

### Phase 6C-5: リリース準備 - 完了 ✅

#### ビルド・配布
- クロスプラットフォーム対応 electron-builder 統合
- macOS (dmg, zip), Windows (nsis, portable), Linux (AppImage, deb) を対象
- electron-updater による自動更新機能対応
- GitHub Releases プロバイダー設定（owner: k-nasa, repo: ajisai）

#### プラットフォーム別セットアップ
- macOS: DMG + ZIP 配布（コード署名スタブ: sign: false）
- Windows: NSIS + ポータブルインストーラー（ワンクリック無効化）
- Linux: AppImage + DEB パッケージサポート

#### 自動更新インフラ
- `electron-updater` が GitHub リリースを監視
- バイナリ比較による効率的な更新
- コンソール ログ出力付きエラーハンドリング
- 開発モードでは自動更新チェックをスキップ

---

## 完了済みフェーズ

### Phase 1-5A: コアエンジン・50+ トランスフォーム ✅
### Phase 5B-5E: 高度なトランスフォーム・スクリプティング ✅
### Phase 6A: ウィンドウ関数・ワークフロー拡張 ✅

---

## リリースノート形式

各リリースでは以下を記録してください:
- **追加されたトランスフォーム**（ある場合）
- **UI 改善**（キャンバス, フォーム, パネル）
- **バグ修正**（該当する issue 番号付き）
- **パフォーマンス改善**（バンドルサイズ, 実行時間）
- **破壊的変更**（該当する場合）

例:
```
## [0.2.0] - YYYY-MM-DD

### 追加
- トランスフォーム別フォーム定義（50+ トランスフォーム）
- Undo/Redo スタック対応

### 修正
- 50+ ノードキャンバスのパフォーマンス改善
- パイプライン変更時の自動保存

### 変更
- バンドルサイズを X kB から Y kB に最適化
```

---

## バージョン番号

- **パッチ (0.x.y)**: バグ修正、マイナー UI 調整
- **マイナー (0.y.0)**: 新規トランスフォーム、新機能
- **メジャー (x.0.0)**: フェーズ完了、アーキテクチャ変更

現在: **0.1.0** — Phase 6（GUI リライト）完了、Phase 7+ 計画準備中

---

## 将来のロードマップ

### Phase 6D: egui GUI 廃止
- レガシー `crates/gui` をワークスペースから除去
- リファレンスブランチにアーカイブ

### Phase 7: フォームビルダー拡張
- カスタムトランスフォームスキーマジェネレータ
- フィールド別検証ルール
- フォームテンプレートライブラリ

### Phase 8: 高度な分析
- パイプライン実行メトリクスダッシュボード
- トランスフォームパフォーマンスプロファイリング
- データ系統追跡（Data Lineage）

---

最終更新: 2026-06-08  
メンテナンス: Claude Code + Kentaro Tanabe
