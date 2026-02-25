# Stage 1: Build the frontend
FROM oven/bun:latest AS frontend-builder
WORKDIR /app/frontend
COPY ai-notes-app/package.json ai-notes-app/bun.lock ./
RUN bun install
COPY ai-notes-app/ .
RUN bun run build

# Stage 2: Build the Go backend
FROM golang:1.23-alpine AS backend-builder
WORKDIR /app/backend
COPY backend-go/go.mod backend-go/go.sum ./
RUN go mod download
COPY backend-go/ .
RUN go build -o main main.go

# Stage 3: Final image
FROM alpine:latest
WORKDIR /app

# Install dependencies if needed (none for static binary)
RUN apk add --no-cache libc6-compat

# Copy built frontend to the 'static' directory served by Go
COPY --from=frontend-builder /app/frontend/build ./static

# Copy built backend
COPY --from=backend-builder /app/backend/main .

# Ensure the 'notes' directory exists or is mounted
RUN mkdir -p notes

# Expose port and set environment variables
EXPOSE 8000
ENV PORT=8000

# Run the application
CMD ["./main"]
