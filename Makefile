.PHONY: help setup start stop logs clean test dev build-backend build-frontend deploy

help:
	@echo "LLM Proxy - Available Commands"
	@echo "================================"
	@echo "make setup          - 初期セットアップ"
	@echo "make start          - サービス起動"
	@echo "make stop           - サービス停止"
	@echo "make logs           - ログ表示"
	@echo "make clean          - データクリア"
	@echo "make test           - テスト実行"
	@echo "make dev            - 開発環境起動"
	@echo "make build-backend  - バックエンドビルド"
	@echo "make build-frontend - フロントエンドビルド"
	@echo "make deploy         - 本番デプロイ"

setup:
	@echo "🚀 Setting up LLM Proxy..."
	@if [ ! -f .env ]; then cp .env.example .env; fi
	@echo "⚠️  Please edit .env and add your API keys"
	docker-compose pull
	@if [ -d frontend ]; then cd frontend && npm install; fi

start:
	@echo "▶️  Starting services..."
	docker-compose up -d
	@echo "✅ Services started!"
	@echo "   - PostgreSQL: http://localhost:5432"
	@echo "   - Qdrant: http://localhost:6333"
	@echo "   - LiteLLM: http://localhost:4000"
	@echo ""
	@echo "To start frontend: cd frontend && npm run dev"

stop:
	@echo "⏹️  Stopping services..."
	docker-compose down

logs:
	docker-compose logs -f

clean:
	@echo "🧹 Cleaning up..."
	docker-compose down -v
	@if [ -d frontend/node_modules ]; then rm -rf frontend/node_modules frontend/.next; fi
	@if [ -d backend/target ]; then rm -rf backend/target; fi

test:
	@echo "🧪 Running tests..."
	@if [ -d backend ]; then cd backend && cargo test; fi
	@if [ -d frontend ]; then cd frontend && npm test; fi

dev:
	@echo "🔧 Starting development environment..."
	docker-compose up -d
	@echo "⏳ Waiting for services..."
	@sleep 5
	@echo "Services ready!"
	@echo ""
	@echo "To start frontend: cd frontend && npm run dev"
	@echo "To start backend: cd backend && cargo run"

build-backend:
	@echo "🔨 Building backend..."
	cd backend && cargo build --release

build-frontend:
	@echo "🔨 Building frontend..."
	cd frontend && npm run build

deploy: build-backend build-frontend
	@echo "🚀 Deploying..."
	docker-compose up -d
	@echo "✅ Deployment complete!"
