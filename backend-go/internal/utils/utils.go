package utils

import (
	"os"
	"path/filepath"
)

func GetEnv(key, fallback string) string {
	if value, ok := os.LookupEnv(key); ok {
		return value
	}
	return fallback
}

func GetPath(path string) string {
	// If it's an absolute path, return as is
	if filepath.IsAbs(path) {
		return path
	}
	// Try current dir
	if _, err := os.Stat(path); err == nil {
		return path
	}
	// Try ../ (for running from subdirectories)
	parentPath := filepath.Join("..", path)
	if _, err := os.Stat(parentPath); err == nil {
		return parentPath
	}
	// Default to current dir if not found anywhere else
	return path
}
