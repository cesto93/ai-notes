package config

import (
	"log"
	"os"

	"github.com/joho/godotenv"
)

func LoadConfig() {
	// Try loading .env from current directory (Docker or local run from root)
	err := godotenv.Load(".env")
	if err != nil {
		// Try parent directory (Running from backend-go subdirectory)
		err = godotenv.Load("../.env")
		if err != nil {
			log.Println("No .env file found, using environment variables")
		}
	}
}

func GetEnv(key, fallback string) string {
	if value, ok := os.LookupEnv(key); ok {
		return value
	}
	return fallback
}
