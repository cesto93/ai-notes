run:
	uv run python -m streamlit run src/streamlit_ui.py
coverage:
	uv run pytest --cov=src --cov-report=html tests/
docker-build:
	docker build -t streamlit_app .
start:
	docker compose up
stop:
	docker compose down
api:
	uv run uvicorn src.api:app --reload --host 0.0.0.0 --port 8000
venv:
	uv venv
