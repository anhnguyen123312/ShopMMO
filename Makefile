# MMO API - Development Makefile

.PHONY: help run build test fmt lint clean docker-build docker-run setup swagger swagger-gen

# Default target
help:
	@echo "MMO API - Available Commands:"
	@echo ""
	@echo "Development:"
	@echo "  make run          - Run the server in development mode"
	@echo "  make build        - Build the project"
	@echo "  make build-release- Build optimized release binary"
	@echo "  make test         - Run all tests"
	@echo "  make test-verbose - Run tests with verbose output"
	@echo ""
	@echo "Code Quality:"
	@echo "  make fmt          - Format code with rustfmt"
	@echo "  make lint         - Run clippy linter"
	@echo "  make check        - Run fmt + lint + test"
	@echo ""
	@echo "Database:"
	@echo "  make mongo-start  - Start MongoDB in Docker"
	@echo "  make mongo-stop   - Stop MongoDB"
	@echo "  make redis-start  - Start Redis in Docker"
	@echo "  make redis-stop   - Stop Redis"
	@echo "  make db-start     - Start MongoDB + Redis"
	@echo "  make db-stop      - Stop MongoDB + Redis"
	@echo ""
	@echo "Swagger/OpenAPI:"
	@echo "  make swagger      - Start Swagger UI server (port 8081)"
	@echo "  make swagger-gen  - Generate OpenAPI JSON to swagger/openapi.json"
	@echo ""
	@echo "Utilities:"
	@echo "  make clean        - Remove build artifacts"
	@echo "  make setup        - Initial project setup"
	@echo "  make logs         - View application logs"
	@echo ""

# Development
run:
	@echo "🚀 Starting MMO API server..."
	cargo run

build:
	@echo "🔨 Building project..."
	cargo build

build-release:
	@echo "🔨 Building release binary..."
	cargo build --release
	@echo "✅ Binary: target/release/mmo-api"

# Testing
test:
	@echo "🧪 Running tests..."
	cargo test

test-verbose:
	@echo "🧪 Running tests (verbose)..."
	cargo test -- --nocapture

# Code Quality
fmt:
	@echo "📝 Formatting code..."
	cargo fmt

lint:
	@echo "🔍 Running clippy..."
	cargo clippy -- -D warnings

check: fmt lint test
	@echo "✅ All checks passed!"

# Database
mongo-start:
	@echo "🗄️  Starting MongoDB..."
	docker run -d --name mmo-mongo \
		-p 27017:27017 \
		-v mmo-mongo-data:/data/db \
		mongo:latest
	@echo "✅ MongoDB started on port 27017"

mongo-stop:
	@echo "🛑 Stopping MongoDB..."
	docker stop mmo-mongo || true
	docker rm mmo-mongo || true

redis-start:
	@echo "📦 Starting Redis..."
	docker run -d --name mmo-redis \
		-p 6379:6379 \
		redis:latest
	@echo "✅ Redis started on port 6379"

redis-stop:
	@echo "🛑 Stopping Redis..."
	docker stop mmo-redis || true
	docker rm mmo-redis || true

db-start: mongo-start redis-start
	@echo "✅ All databases started"

db-stop: mongo-stop redis-stop
	@echo "✅ All databases stopped"

# Utilities
clean:
	@echo "🧹 Cleaning build artifacts..."
	cargo clean
	@echo "✅ Cleaned"

setup:
	@echo "⚙️  Setting up project..."
	@cp -n .env.example .env 2>/dev/null || echo ".env already exists"
	@cargo build
	@echo "✅ Setup complete! Run 'make run' to start the server"

logs:
	@echo "📋 Recent logs:"
	@tail -f /tmp/mmo-api.log 2>/dev/null || echo "No log file found. Run the server with: RUST_LOG=debug cargo run > /tmp/mmo-api.log"

# Swagger/OpenAPI
swagger:
	@echo "📚 Starting Swagger UI server..."
	cargo run --bin swagger_server

swagger-gen:
	@echo "📝 Generating OpenAPI specification..."
	cargo run --bin generate_openapi
	@echo "✅ OpenAPI spec: swagger/openapi.json"

# Docker
docker-build:
	@echo "🐳 Building Docker image..."
	docker build -t mmo-api:latest .

docker-run:
	@echo "🐳 Running Docker container..."
	docker run -d --name mmo-api \
		-p 8080:8080 \
		--env-file .env \
		mmo-api:latest

docker-stop:
	@echo "🛑 Stopping Docker container..."
	docker stop mmo-api || true
	docker rm mmo-api || true
