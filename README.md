# LLM Proxy Web System

企業向けセキュアLLMプロキシシステム - PII保護、RAG統合、完全監査ログ付き

## 🌟 特徴

- **🔒 PII保護**: 会社名・人名・住所を自動検出してマスキング/復元
- **🔍 RAG統合**: ベクトル検索による関連情報の自動追加
- **📝 完全監査**: 全プロンプト（入力・マスク・RAG・出力）を記録
- **🚀 高性能**: Rustバックエンドによる低レイテンシ
- **🎨 モダンUI**: Next.js 14 + TypeScript + Tailwind CSS
- **🤖 マルチモデル**: Claude (Anthropic) / GPT-4 (OpenAI) 対応

## 🏗️ アーキテクチャ

```
┌─────────────────┐
│  Next.js UI     │
│  (Port 3000)    │
└────────┬────────┘
         │ HTTP
┌────────▼────────┐
│  Rust Proxy     │
│  (Port 8080)    │
│  ├─ PII Filter  │
│  ├─ RAG Engine  │
│  └─ Logger      │
└─┬──────────┬────┘
  │          │
  │  ┌───────▼──────┐
  │  │   Qdrant     │
  │  │ Vector Store │
  │  │ (Port 6334)  │
  │  └──────────────┘
  │
  │  ┌──────────────┐
  │  │  PostgreSQL  │
  │  │   Logging    │
  │  │ (Port 5432)  │
  │  └──────────────┘
  │
┌─▼──────────────┐
│   LiteLLM      │
│  (Port 4000)   │
└─┬──────────────┘
  │
  ├─► Claude (Anthropic)
  └─► GPT-4 (OpenAI)
```

## 📋 必要要件

- Docker & Docker Compose
- Rust 1.75+ (開発時のみ)
- Node.js 20+ (開発時のみ)
- OpenAI API Key
- Anthropic API Key

## 🚀 クイックスタート

### 1. リポジトリクローン

```bash
git clone https://github.com/yourusername/llm-proxy-web.git
cd llm-proxy-web
```

### 2. 環境変数設定

```bash
cp .env.example .env
# .envファイルを編集してAPIキーを設定
```

### 3. Docker起動

```bash
make start
# または
docker-compose up -d
```

### 4. フロントエンド起動（開発モード）

```bash
cd frontend
npm install
npm run dev
```

### 5. ブラウザでアクセス

```
http://localhost:3000
```

## 🔧 開発環境セットアップ

### Makeコマンド使用

```bash
make setup    # 初期セットアップ
make start    # サービス起動
make dev      # 開発環境起動
make stop     # サービス停止
make logs     # ログ表示
make clean    # データクリア
make test     # テスト実行
```

### バックエンド（Rust）

```bash
cd backend
cargo build
cargo run
```

### フロントエンド（Next.js）

```bash
cd frontend
npm install
npm run dev
```

## 📚 API エンドポイント

### チャット
- `POST /api/v1/chat/completions` - チャット送信
- `POST /api/v1/chat/stream` - ストリーミングチャット

### モデル管理
- `GET /api/v1/models` - 利用可能なモデル一覧

### RAGドキュメント
- `POST /api/v1/documents` - ドキュメント追加
- `GET /api/v1/documents` - ドキュメント一覧

### ログ
- `GET /api/v1/logs` - ログ検索・取得

### ヘルスチェック
- `GET /api/health` - サーバー状態確認

## 🔐 セキュリティ

- **PII検出**: 正規表現ベースの固有表現認識
- **マスキング**: 一時的なプレースホルダーでPII保護
- **復元**: レスポンス時に元の情報を復元
- **監査ログ**: PostgreSQLに全トランザクション記録

## 🗄️ データベーススキーマ

```sql
CREATE TABLE prompt_logs (
    id UUID PRIMARY KEY,
    timestamp TIMESTAMPTZ NOT NULL,
    original_input TEXT NOT NULL,      -- 元の入力
    masked_input TEXT NOT NULL,        -- マスク済み入力
    rag_context TEXT,                  -- RAGコンテキスト
    llm_output TEXT NOT NULL,          -- LLM生出力
    final_output TEXT NOT NULL,        -- 最終出力（復元済み）
    pii_mappings JSONB NOT NULL        -- PIIマッピング
);
```

## 🧪 テスト

### バックエンド

```bash
cd backend
cargo test
```

### フロントエンド

```bash
cd frontend
npm test
```

## 📦 本番デプロイ

### Docker Composeで完全起動

```bash
docker-compose up -d
```

### 個別ビルド

```bash
# バックエンド
cd backend
cargo build --release

# フロントエンド
cd frontend
npm run build
npm start
```

## 🛠️ カスタマイズ

### PII検出パターン追加

`backend/src/filters/pii_detector.rs`を編集:

```rust
static CUSTOM_PATTERN: Lazy<Regex> = Lazy::new(|| {
    Regex::new(r"your-pattern-here").unwrap()
});
```

### RAG検索設定

`backend/src/rag/mod.rs`でtop_kやembeddingモデルを変更

### UIテーマ

`frontend/tailwind.config.ts`でTailwindテーマをカスタマイズ

## 📊 モニタリング

ログはPostgreSQLに保存され、Web UIから検索可能:

- 日時範囲フィルタ
- キーワード検索
- PII検出件数表示
- ページネーション

## 🤝 コントリビューション

1. Fork the repository
2. Create your feature branch (`git checkout -b feature/amazing-feature`)
3. Commit your changes (`git commit -m 'Add amazing feature'`)
4. Push to the branch (`git push origin feature/amazing-feature`)
5. Open a Pull Request

## 📄 ライセンス

MIT License - 詳細は [LICENSE](LICENSE) を参照

## 🙏 謝辞

- [LiteLLM](https://github.com/BerriAI/litellm)
- [Qdrant](https://qdrant.tech/)
- [FastEmbed](https://github.com/Anush008/fastembed-rs)
- [Next.js](https://nextjs.org/)
- [Axum](https://github.com/tokio-rs/axum)

## 📞 サポート

問題が発生した場合:

1. [Issues](https://github.com/yourusername/llm-proxy-web/issues)を確認
2. 新しいIssueを作成
3. ログを添付してください

---

Made with ❤️ for Enterprise AI
