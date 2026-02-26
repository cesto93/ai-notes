run:
	@echo "Starting backend and frontend..."
	(trap 'kill 0' SIGINT; make api-go & make frontend)

api-go:
	cd backend-go && go run main.go

frontend:
	cd ai-notes-app-tauri && bun run tauri dev

coverage:
	uv run pytest --cov=src --cov-report=html tests/

docker-build:
	docker build -t ai-notes-app .

start:
	docker compose up

stop:
	docker compose down

venv:
	uv venv

format:
	ruff format
