# LLM Proxy - Docker完全ガイド

完全にDockerで動作するLLMプロキシシステム。全コンポーネントがコンテナ化されています。

## 🐳 システム構成

すべてDockerコンテナで実行：

```
┌─────────────────────────────────────────┐
│         Docker Compose                  │
├─────────────────────────────────────────┤
│                                         │
│  📦 frontend (Next.js)      :3000      │
│  📦 backend (Rust)          :8080      │
│  📦 litellm                 :4000      │
│  📦 qdrant                  :6333,6334 │
│  📦 postgres                :5432      │
│                                         │
└─────────────────────────────────────────┘
```

## 🚀 クイックスタート（3ステップ）

### 1. 環境変数設定

```bash
cp .env.example .env
```

`.env`を編集してAPIキーを設定：

```bash
OPENAI_API_KEY=sk-...
ANTHROPIC_API_KEY=sk-ant-...
LITELLM_MASTER_KEY=your-secure-key
```

### 2. Docker起動

**方法A: 自動起動スクリプト（推奨）**

```bash
./start.sh
```

**方法B: Makeコマンド**

```bash
make docker-up
```

**方法C: Docker Composeコマンド**

```bash
docker-compose up -d
```

### 3. ブラウザでアクセス

```
http://localhost:3000
```

## 📋 前提条件

- Docker 20.10+
- Docker Compose 2.0+
- 8GB以上のRAM推奨

## 🎯 コマンド一覧

### 基本操作

```bash
# 全サービス起動
make docker-up
# または
./start.sh

# 全サービス停止
make docker-down

# ログ表示
make docker-logs

# 再ビルド&起動
make docker-rebuild
```

### 開発モード

インフラのみDockerで起動し、フロントエンド・バックエンドはローカルで実行：

```bash
# インフラ起動
make dev

# 別ターミナルでバックエンド起動
cd backend
cargo run

# さらに別ターミナルでフロントエンド起動
cd frontend
npm install
npm run dev
```

### メンテナンス

```bash
# データ完全削除
make clean

# テスト実行
make test

# コンテナ状態確認
docker-compose ps

# 特定サービスのログ
docker-compose logs -f backend
docker-compose logs -f frontend
```

## 🔧 トラブルシューティング

### ポート衝突

ポートが既に使用されている場合：

```bash
# 使用中のポート確認
lsof -i :3000
lsof -i :8080
lsof -i :4000

# docker-compose.ymlでポート変更
ports:
  - "3001:3000"  # 外部ポートを変更
```

### ビルドエラー

```bash
# キャッシュクリア&再ビルド
docker-compose build --no-cache
docker-compose up -d
```

### データベース接続エラー

```bash
# PostgreSQL起動確認
docker-compose logs postgres

# データベース再作成
docker-compose down -v
docker-compose up -d
```

### メモリ不足

```bash
# Dockerリソース確認
docker stats

# 不要なイメージ削除
docker system prune -a
```

## 📊 各サービスの詳細

### フロントエンド (port 3000)

- Next.js 14 + TypeScript
- Tailwind CSS
- ビルド時間: 約3-5分

```bash
# フロントエンドのみ再起動
docker-compose restart frontend

# フロントエンドログ
docker-compose logs -f frontend
```

### バックエンド (port 8080)

- Rust + Axum
- ビルド時間: 約10-15分（初回）
- ヘルスチェック: http://localhost:8080/api/health

```bash
# バックエンドのみ再起動
docker-compose restart backend

# バックエンドログ
docker-compose logs -f backend
```

### LiteLLM (port 4000)

- マルチLLMプロキシ
- Claude + GPT-4対応
- ヘルスチェック: http://localhost:4000/health

```bash
# LiteLLMのみ再起動
docker-compose restart litellm
```

### Qdrant (port 6333, 6334)

- ベクトルデータベース
- Web UI: http://localhost:6333/dashboard

```bash
# Qdrant Web UI アクセス
open http://localhost:6333/dashboard
```

### PostgreSQL (port 5432)

- ログ保存用DB
- 自動マイグレーション

```bash
# PostgreSQL接続
docker exec -it llm-proxy-postgres psql -U llmproxy -d llm_proxy

# テーブル確認
\dt

# ログ件数確認
SELECT COUNT(*) FROM prompt_logs;
```

## 🔐 セキュリティ

### 本番環境での推奨設定

1. **環境変数を安全に管理**

```bash
# .envを.gitignoreに追加（既に設定済み）
echo ".env" >> .gitignore
```

2. **パスワード変更**

`docker-compose.yml`のPostgreSQLパスワードを変更：

```yaml
environment:
  POSTGRES_PASSWORD: your-secure-password-here
```

3. **外部公開時の設定**

```yaml
# ポート制限（外部からアクセスさせない）
ports:
  - "127.0.0.1:5432:5432"  # localhostのみ
```

## 📈 パフォーマンス最適化

### ビルド時間短縮

```bash
# マルチステージビルドのキャッシュ利用
docker-compose build --parallel
```

### リソース制限

`docker-compose.yml`に追加：

```yaml
services:
  backend:
    deploy:
      resources:
        limits:
          cpus: '2'
          memory: 4G
```

## 🌐 本番デプロイ

### Docker Swarm

```bash
# Swarmモード有効化
docker swarm init

# スタックデプロイ
docker stack deploy -c docker-compose.yml llm-proxy
```

### Kubernetes

```bash
# Komposeでマニフェスト生成
kompose convert -f docker-compose.yml

# デプロイ
kubectl apply -f .
```

## 📝 ログ管理

### ログローテーション

`docker-compose.yml`に追加：

```yaml
services:
  backend:
    logging:
      driver: "json-file"
      options:
        max-size: "10m"
        max-file: "3"
```

### ログ集約

```bash
# 全サービスのログを出力
docker-compose logs > all-logs.txt

# エラーログのみ
docker-compose logs 2>&1 | grep -i error
```

## 🔄 アップデート

### フロントエンド/バックエンドのコード変更時（必須）

UIやAPIに変更を加えた場合、必ず以下の手順でDockerコンテナを再ビルド・再起動してください。
コードを編集しただけではブラウザに反映されません。

```bash
# キャッシュクリア＆再ビルド＆再起動（推奨）
docker compose up --build -d
```

変更が反映されない場合はキャッシュを完全にクリアしてください：

```bash
# フロントエンドのキャッシュクリア＆再ビルド
docker compose build --no-cache frontend
docker compose up -d frontend

# バックエンドのキャッシュクリア＆再ビルド
docker compose build --no-cache backend
docker compose up -d backend

# 全サービス一括
docker compose build --no-cache
docker compose up -d
```

### イメージ更新

```bash
# 最新イメージ取得
docker-compose pull

# 再起動
docker-compose up -d
```

### コード更新

```bash
# 再ビルド
make docker-rebuild
```

## ❓ FAQ

**Q: 初回起動が遅い**
A: Rustバックエンドのビルドに10-15分かかります。2回目以降は数秒で起動します。

**Q: メモリ使用量が多い**
A: 全コンテナで約4-6GB使用します。不要な時は`docker-compose down`で停止してください。

**Q: データはどこに保存される？**
A: Dockerボリュームに保存されます。`docker volume ls`で確認できます。

**Q: データを完全に削除したい**
A: `make clean`または`docker-compose down -v`を実行してください。

## 📞 サポート

問題が発生した場合：

1. ログを確認: `make docker-logs`
2. サービス状態確認: `docker-compose ps`
3. ヘルスチェック: `curl http://localhost:8080/api/health`
4. GitHubでIssue作成

---

Made with ❤️ using Docker
