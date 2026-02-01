# ワードウルフ - マルチプレイヤーゲーム

Rustで実装された完全自作のウェブサーバーを使用したワードウルフゲームです。外部のウェブフレームワークを使わず、RustとHTML/JavaScriptのみで構築されています。

## プロジェクト概要

ワードウルフは、プレイヤーが似たようなテーマを受け取り、少数派（ワードウルフ）を見つけ出す社会的推理ゲームです。

### 特徴

- 🦀 **Pure Rust バックエンド** - 外部依存なし、標準ライブラリのみ使用
- 🌐 **カスタムHTTPサーバー** - フレームワーク不要の完全自作実装
- ⚡ **リアルタイム通信** - Server-Sent Events (SSE) によるライブアップデート
- 🎮 **完全な日本語対応** - UI、チャット、プレイヤー名すべてUTF-8対応
- ⏱️ **自動タイマーシステム** - ディスカッション時間制限と自動投票移行
- 🔒 **スレッドセーフ** - Arc<Mutex<T>>による並行アクセス管理

## 技術スタック

### バックエンド

- **言語**: Rust (Edition 2024)
- **依存関係**: なし（標準ライブラリのみ）
- **ウェブサーバー**: カスタム実装（`std::net::TcpListener`）
- **並行処理**: `std::sync::{Arc, Mutex}`, `std::sync::mpsc`
- **リアルタイム通信**: Server-Sent Events (SSE)

### フロントエンド

- **HTML5** - セマンティックなマークアップ
- **JavaScript (Vanilla)** - フレームワーク不要
- **CSS** - シンプルなスタイリング
- **EventSource API** - SSEクライアント

### 開発環境

- **Nix Flakes** - 再現可能な開発環境
- **Rust Toolchain**: stable + rust-src + rust-analyzer

## システムアーキテクチャ

### レイヤー構造

プロジェクトは3層アーキテクチャで構成されています：

```
┌─────────────────────────────────────────┐
│         Network Layer (HTTP/SSE)        │  ← クライアントとの通信
├─────────────────────────────────────────┤
│       Service Layer (RoomManager)       │  ← ビジネスロジックの調整
├─────────────────────────────────────────┤
│      Domain Layer (Game/Player/Room)    │  ← コアゲームロジック
└─────────────────────────────────────────┘
```

### ディレクトリ構造

```
src/
├── main.rs                 # エントリーポイント、HTTPサーバー起動
├── types.rs                # 共通型定義（Role, ThemeGenre, RoomConfig）
├── game/                   # ドメイン層：ゲームロジック
│   ├── mod.rs              # モジュール公開
│   ├── player.rs           # プレイヤー状態管理
│   ├── state.rs            # ゲーム状態機械（FSM）
│   ├── theme.rs            # テーマデータベース
│   └── rules.rs            # ゲームルール（投票集計、勝利判定）
├── rooms/                  # サービス層：ルーム管理
│   ├── mod.rs              # モジュール公開
│   ├── room.rs             # 単一ルームのライフサイクル管理
│   └── manager.rs          # 複数ルームの並行管理（Arc<Mutex<>>）
└── network/                # ネットワーク層：HTTP/SSE
    ├── mod.rs              # モジュール公開
    ├── http.rs             # HTTPリクエスト/レスポンス解析
    ├── sse.rs              # Server-Sent Events実装
    └── handlers.rs         # HTTPエンドポイント処理
```

### コア技術コンセプト

#### 1. **完全自作HTTPサーバー**

外部クレートを使わず、`std::net::TcpListener`で実装：

```rust
let listener = TcpListener::bind("127.0.0.1:8080")?;

for stream in listener.incoming() {
    let stream = stream?;
    let room_manager = room_manager.clone();

    thread::spawn(move || {
        handle_connection(stream, room_manager);
    });
}
```

- 各接続を新しいスレッドで処理
- HTTPリクエストを手動でパース
- レスポンスを手動で構築

#### 2. **Server-Sent Events (SSE) によるリアルタイム更新**

WebSocketではなくSSEを選択した理由：
- シンプルな実装（HTTP上のテキストストリーム）
- 一方向通信で十分（サーバー→クライアント）
- 自動再接続機能内蔵

実装方法：
```rust
// バックエンド：mpsc channelでイベント送信
let (tx, rx) = mpsc::channel();
room.add_sender(tx);

// フロントエンド：EventSourceで受信
const eventSource = new EventSource('/events?room_id=abc');
eventSource.onmessage = (event) => {
    console.log(event.data);
};
```

#### 3. **スレッドセーフな状態管理（Arc<Mutex<T>>）**

複数のスレッドから安全にルームにアクセス：

```rust
pub type SharedRooms = Arc<Mutex<HashMap<RoomId, Room>>>;

pub struct RoomManager {
    rooms: SharedRooms,
}

impl RoomManager {
    pub fn with_room<F, R>(&self, room_id: &RoomId, f: F) -> Result<R, String>
    where
        F: FnOnce(&mut Room) -> Result<R, String>,
    {
        let mut rooms = self.rooms.lock().unwrap();
        let room = rooms.get_mut(room_id)
            .ok_or("Room not found")?;
        f(room)
    }
}
```

- `Arc`: 複数スレッド間でデータを共有
- `Mutex`: 排他制御（一度に1スレッドのみアクセス可能）
- クロージャパターン: ロック保持中に操作を実行

#### 4. **状態機械パターン（Finite State Machine）**

ゲームの状態遷移を型で保証：

```rust
pub enum GameState {
    Lobby { ready_players: HashSet<PlayerId> },
    ThemeSubmission { confirmed_players: HashSet<PlayerId> },
    Discussion { time_remaining: Option<u32> },
    Voting { voted_players: HashSet<PlayerId> },
    Finished { citizens_won: bool, wolves: Vec<PlayerId> },
}
```

不正な遷移（例：Lobby → Voting）はコンパイルエラーになる設計。

#### 5. **UTF-8マルチバイト文字対応**

日本語はUTF-8で3バイト（例：`あ` = `%E3%81%82`）：

```rust
fn url_decode(s: &str) -> String {
    // %E3%81%82 → [0xE3, 0x81, 0x82] → "あ"
    let extra_bytes = if byte >= 0xF0 { 3 }      // 4バイトUTF-8
                      else if byte >= 0xE0 { 2 }  // 3バイトUTF-8（日本語）
                      else if byte >= 0xC0 { 1 }  // 2バイトUTF-8
                      else { 0 };                 // 1バイト（ASCII）

    // 追加バイトを収集してUTF-8文字列に変換
    String::from_utf8(bytes).unwrap()
}
```

#### 6. **バックグラウンドタイマースレッド**

自動投票移行のための定期チェック：

```rust
{
    let timer_manager = room_manager.clone();
    thread::spawn(move || {
        loop {
            thread::sleep(Duration::from_secs(1)); // 1秒ごと
            timer_manager.check_all_timers();       // 全ルームをチェック
        }
    });
}
```

## ゲームフロー

### 1. **ログイン** (`login.html`)
プレイヤー名を入力

### 2. **ホーム画面** (`home.html`)
- ルーム作成
- ルーム参加

### 3. **ルーム作成** (`room_create.html`)
設定項目：
- ルーム名
- 最大プレイヤー数（3-10人）
- ワードウルフ数（少数派を保つ制限あり）
- テーマジャンル（食べ物、動物、場所、物）
- ディスカッション時間（2-5分）

### 4. **待機室** (`stay.html`)
- 他のプレイヤーを待つ
- 全員が準備完了でゲーム開始

### 5. **テーマ確認** (`theme.html`)
- 自分のテーマとロール（市民/ワードウルフ）を確認
- 全員確認完了でディスカッション開始

### 6. **ゲーム画面** (`game.html`)

#### ディスカッションフェーズ
- カウントダウンタイマー表示
- リアルタイムチャット（日本語対応）
- 参加者一覧
- タイマー終了で自動的に投票フェーズへ

#### 投票フェーズ
- 参加者一覧から1人を選択
- 全員投票完了で即座に結果発表

### 7. **結果発表**
- 脱落者が**ワードウルフ** → **市民の勝利**
- 脱落者が**市民** → **ワードウルフの勝利**

**重要**: ワードウルフは**1回の投票で終了**（複数ラウンドなし）

## APIエンドポイント

### ルーム管理
- `POST /room/create` - ルーム作成
- `POST /room/join` - ルーム参加
- `GET /room/list` - ルーム一覧取得
- `GET /room/state` - ルーム状態取得
- `GET /room/players` - プレイヤー一覧取得

### ゲーム進行
- `POST /room/ready` - プレイヤー準備完了
- `POST /room/theme/confirm` - テーマ確認完了
- `GET /room/timer` - 残り時間取得
- `POST /room/start-vote` - 投票開始（タイマー自動実行）
- `POST /room/vote` - 投票実行

### チャット
- `POST /room/chat` - チャットメッセージ送信

### リアルタイム更新
- `GET /events?room_id=xxx` - SSE接続（ゲーム状態の購読）

### プレイヤー情報
- `GET /player/theme?room_id=xxx&player_id=yyy` - 自分のテーマ取得

## セットアップと実行

### 必要要件

- Nix（推奨）または Rust stable toolchain

### Nixを使う場合（推奨）

```bash
# 開発環境に入る
nix develop

# ビルド
cargo build

# 実行
cargo run

# テスト
cargo test
```

### Nixなしの場合

```bash
# Rustがインストールされている必要があります
# https://rustup.rs/

# ビルド
cargo build --release

# 実行
cargo run --release
```

### サーバー起動後

ブラウザで `http://127.0.0.1:8080` にアクセス

## 設計上の特徴

### 1. **型駆動開発**
- 不正な状態を型システムで防ぐ
- `Option<T>`と`Result<T, E>`で明示的なエラー処理
- Enumで限定的な選択肢を表現

### 2. **純粋関数**
- ゲームロジック（`rules.rs`）は副作用なし
- テスト容易性が高い
- 予測可能な動作

### 3. **レイヤードアーキテクチャ**
- 各層は下層のみに依存
- ドメインロジックはネットワーク層から独立
- 変更の影響範囲が限定的

### 4. **並行性の安全性**
- Rustの所有権システムでデータ競合を防ぐ
- `Arc<Mutex<T>>`で明示的な共有状態管理
- コンパイル時に並行性のバグを検出

### 5. **ドメイン駆動設計（DDD）**
- `Player`, `Room`, `GameState`などのドメインモデル
- ビジネスロジックをドメイン層に集約
- インフラ（HTTP）とドメインの分離

## テストカバレッジ

全35個のユニットテストが含まれています：

```bash
cargo test

# 出力例：
# test result: ok. 35 passed; 0 failed; 0 ignored; 0 measured
```

テスト範囲：
- ゲームルール（投票集計、勝利判定、ロール割り当て）
- プレイヤー状態管理
- ルーム管理（作成、参加、削除）
- 状態機械の遷移
- HTTP解析
- URLデコーディング

## パフォーマンス特性

- **メモリ効率**: Rustのゼロコスト抽象化
- **並行処理**: スレッドプールなしでも数百接続対応可能
- **低レイテンシ**: SSEによるリアルタイム更新（遅延<100ms）
- **スケーラビリティ**: Arc<Mutex<>>でCPUコア数分並列化

## セキュリティ考慮事項

現在の実装は**学習/デモ目的**です。本番環境では以下の追加が必要：

- [ ] HTTPS/TLS対応
- [ ] 認証・認可システム
- [ ] レート制限
- [ ] 入力バリデーション強化
- [ ] XSS/CSRF対策
- [ ] セッション管理

## 既知の制限事項

- 永続化なし（サーバー再起動でデータ消失）
- 水平スケーリング不可（単一プロセス）
- WebSocket非対応（SSEのみ）
- ルーム自動削除機能なし

## 今後の拡張案

- [ ] データベース統合（SQLite/PostgreSQL）
- [ ] WebSocket対応
- [ ] ルーム履歴・統計
- [ ] ランキングシステム
- [ ] カスタムテーマ作成UI
- [ ] リプレイ機能

## ライセンス

このプロジェクトは学習目的で作成されました。

## 貢献

プルリクエストは歓迎します！大きな変更の場合は、まずissueで議論してください。

## 学習リソース

このプロジェクトで使われている技術を学ぶには：

- **Rust基礎**: [The Rust Programming Language](https://doc.rust-lang.org/book/)
- **並行処理**: [Rust Atomics and Locks](https://marabos.nl/atomics/)
- **ネットワーク**: [TCP/IP Illustrated](https://en.wikipedia.org/wiki/TCP/IP_Illustrated)
- **アーキテクチャ**: [Domain-Driven Design](https://www.oreilly.com/library/view/domain-driven-design-tackling/0321125215/)

---

**開発**: ne-pro-group
**技術スタック**: 🦀 Rust + 🌐 Vanilla JS + 📡 SSE
