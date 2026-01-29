# Stage 1: Build the frontend
FROM oven/bun:latest AS frontend-builder
WORKDIR /app/frontend
COPY ai-notes-app/package.json ai-notes-app/bun.lock ./
RUN bun install
COPY ai-notes-app/ .
RUN bun run build

# Stage 2: Final image
FROM python:3.12-slim
WORKDIR /app

# Install uv installer
COPY --from=ghcr.io/astral-sh/uv:latest /uv /uv/bin/uv

# Install system dependencies if any (none needed for now based on pyproject.toml)

# Install Python dependencies
COPY pyproject.toml uv.lock ./
RUN /uv/bin/uv sync --frozen --no-cache

# Copy application code
COPY src ./src
COPY main.py .

# Copy built frontend to the 'static' directory served by FastAPI
COPY --from=frontend-builder /app/frontend/build ./static

# Ensure the 'notes' directory exists or is mounted
RUN mkdir -p notes

# Expose port and set environment variables
EXPOSE 8000
ENV DOCKER_MODE=1
ENV PYTHONUNBUFFERED=1

# Run the application
CMD ["/uv/bin/uv", "run", "uvicorn", "src.api:app", "--host", "0.0.0.0", "--port", "8000"]
