.PHONY: help setup start stop logs clean test dev build-backend build-frontend deploy docker-up docker-down docker-logs docker-rebuild

help:
	@echo "LLM Proxy - Available Commands"
	@echo "================================"
	@echo "Docker Commands (推奨):"
	@echo "  make docker-up       - 全サービスをDockerで起動"
	@echo "  make docker-down     - 全サービスを停止"
	@echo "  make docker-logs     - ログ表示"
	@echo "  make docker-rebuild  - 再ビルド&起動"
	@echo ""
	@echo "開発環境:"
	@echo "  make dev             - インフラのみDocker起動（開発用）"
	@echo "  make setup           - 初期セットアップ"
	@echo ""
	@echo "その他:"
	@echo "  make clean           - データクリア"
	@echo "  make test            - テスト実行"

setup:
	@echo "🚀 Setting up LLM Proxy..."
	@if [ ! -f .env ]; then cp .env.example .env; fi
	@echo "⚠️  Please edit .env and add your API keys"

# Docker完全起動（本番・デモ用）
docker-up:
	@echo "🚀 Starting all services with Docker..."
	@chmod +x start.sh
	@./start.sh

docker-down:
	@echo "⏹️  Stopping all Docker services..."
	@docker-compose down

docker-logs:
	@docker-compose logs -f

docker-rebuild:
	@echo "🔨 Rebuilding and restarting..."
	@docker-compose down
	@docker-compose build
	@docker-compose up -d

# 開発環境（インフラのみDocker）
dev:
	@echo "🔧 Starting infrastructure services..."
	@docker-compose -f docker-compose.dev.yml up -d
	@echo "⏳ Waiting for services..."
	@sleep 5
	@echo ""
	@echo "✅ Infrastructure ready!"
	@echo ""
	@echo "次のステップ:"
	@echo "  1. バックエンド: cd backend && cargo run"
	@echo "  2. フロントエンド: cd frontend && npm run dev"
	@echo ""

# 旧コマンド（互換性のため残す）
start: docker-up

stop: docker-down

logs: docker-logs

clean:
	@echo "🧹 Cleaning up..."
	@docker-compose down -v
	@docker-compose -f docker-compose.dev.yml down -v
	@if [ -d frontend/node_modules ]; then rm -rf frontend/node_modules frontend/.next; fi
	@if [ -d backend/target ]; then rm -rf backend/target; fi

test:
	@echo "🧪 Running tests..."
	@if [ -d backend ]; then cd backend && cargo test; fi
	@if [ -d frontend ]; then cd frontend && npm test; fi

build-backend:
	@echo "🔨 Building backend..."
	@cd backend && cargo build --release

build-frontend:
	@echo "🔨 Building frontend..."
	@cd frontend && npm run build

deploy: docker-rebuild
	@echo "✅ Deployment complete!"
