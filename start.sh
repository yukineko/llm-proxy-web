#!/bin/bash

set -e

# カラー定義
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
RED='\033[0;31m'
NC='\033[0m'

echo -e "${BLUE}========================================${NC}"
echo -e "${BLUE}  LLM Proxy - Docker起動スクリプト${NC}"
echo -e "${BLUE}========================================${NC}"
echo ""

# .envファイルの確認
if [ ! -f .env ]; then
    echo -e "${YELLOW}⚠️  .envファイルが見つかりません${NC}"
    echo -e "${BLUE}📝 .env.exampleから.envを作成します...${NC}"
    cp .env.example .env
    echo -e "${RED}❗ 重要: .envファイルを編集してAPIキーを設定してください！${NC}"
    echo ""
    echo -e "${YELLOW}設定が必要な項目:${NC}"
    echo "  - OPENAI_API_KEY"
    echo "  - ANTHROPIC_API_KEY"
    echo "  - LITELLM_MASTER_KEY"
    echo ""
    read -p "続行しますか？ (y/N): " confirm
    if [ "$confirm" != "y" ] && [ "$confirm" != "Y" ]; then
        echo -e "${RED}❌ キャンセルしました${NC}"
        exit 1
    fi
fi

# APIキーの確認
source .env
if [ -z "$OPENAI_API_KEY" ] || [ "$OPENAI_API_KEY" = "sk-your-openai-key-here" ]; then
    echo -e "${RED}❌ OPENAI_API_KEYが設定されていません${NC}"
    exit 1
fi

if [ -z "$ANTHROPIC_API_KEY" ] || [ "$ANTHROPIC_API_KEY" = "sk-ant-your-anthropic-key-here" ]; then
    echo -e "${RED}❌ ANTHROPIC_API_KEYが設定されていません${NC}"
    exit 1
fi

echo -e "${GREEN}✅ 環境変数確認完了${NC}"
echo ""

# 既存のコンテナを停止
echo -e "${BLUE}🛑 既存のコンテナを停止中...${NC}"
docker-compose down 2>/dev/null || true

# イメージのビルド
echo -e "${BLUE}🔨 Dockerイメージをビルド中...${NC}"
docker-compose build

# コンテナの起動
echo -e "${BLUE}🚀 コンテナを起動中...${NC}"
docker-compose up -d

# 起動待機
echo -e "${BLUE}⏳ サービスの起動を待機中...${NC}"
sleep 5

# ヘルスチェック
echo ""
echo -e "${BLUE}🔍 サービス状態を確認中...${NC}"
echo ""

check_service() {
    local service=$1
    local url=$2
    local name=$3
    
    if curl -s -f "$url" > /dev/null 2>&1; then
        echo -e "  ${GREEN}✅ $name: 正常${NC}"
        return 0
    else
        echo -e "  ${RED}❌ $name: エラー${NC}"
        return 1
    fi
}

check_service "postgres" "http://localhost:5432" "PostgreSQL" || true
check_service "qdrant" "http://localhost:6333/health" "Qdrant"
check_service "litellm" "http://localhost:4000/health" "LiteLLM"

# バックエンドの起動を待つ
echo ""
echo -e "${BLUE}⏳ バックエンドの起動を待機中...${NC}"
for i in {1..30}; do
    if curl -s -f http://localhost:8080/api/health > /dev/null 2>&1; then
        echo -e "${GREEN}✅ バックエンド: 起動完了${NC}"
        break
    fi
    echo -n "."
    sleep 2
done
echo ""

# フロントエンドの起動を待つ
echo -e "${BLUE}⏳ フロントエンドの起動を待機中...${NC}"
for i in {1..30}; do
    if curl -s -f http://localhost:3000 > /dev/null 2>&1; then
        echo -e "${GREEN}✅ フロントエンド: 起動完了${NC}"
        break
    fi
    echo -n "."
    sleep 2
done
echo ""

# 完了メッセージ
echo ""
echo -e "${GREEN}========================================${NC}"
echo -e "${GREEN}  🎉 起動完了！${NC}"
echo -e "${GREEN}========================================${NC}"
echo ""
echo -e "${BLUE}📍 アクセス先:${NC}"
echo -e "  ${GREEN}🌐 フロントエンド:${NC}  http://localhost:3000"
echo -e "  ${GREEN}⚙️  バックエンド:${NC}    http://localhost:8080"
echo -e "  ${GREEN}🔍 Qdrant:${NC}         http://localhost:6333"
echo -e "  ${GREEN}🤖 LiteLLM:${NC}        http://localhost:4000"
echo ""
echo -e "${BLUE}📝 コマンド:${NC}"
echo -e "  ${YELLOW}ログ表示:${NC}       docker-compose logs -f"
echo -e "  ${YELLOW}停止:${NC}           docker-compose down"
echo -e "  ${YELLOW}再起動:${NC}         docker-compose restart"
echo -e "  ${YELLOW}完全削除:${NC}       docker-compose down -v"
echo ""
echo -e "${GREEN}✨ ブラウザで http://localhost:3000 を開いてください！${NC}"
echo ""
