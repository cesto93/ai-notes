package ai

import (
	"context"
	"fmt"
	"os"

	"github.com/google/generative-ai-go/genai"
	"github.com/pier/ai-notes/backend-go/internal/models"
	"google.golang.org/api/option"
)

type AIService interface {
	Summarize(ctx context.Context, text string) (string, error)
	Paraphrase(ctx context.Context, text string) (string, error)
	GenerateMindmap(ctx context.Context, text string) (string, error)
}

type GeminiService struct {
	client *genai.Client
	model  string
}

func NewGeminiService(ctx context.Context, modelName string) (*GeminiService, error) {
	apiKey := os.Getenv("GOOGLE_API_KEY")
	client, err := genai.NewClient(ctx, option.WithAPIKey(apiKey))
	if err != nil {
		return nil, err
	}
	return &GeminiService{client: client, model: modelName}, nil
}

func (s *GeminiService) Summarize(ctx context.Context, text string) (string, error) {
	model := s.client.GenerativeModel(s.model)
	resp, err := model.GenerateContent(ctx, genai.Text("Summarize the following note in a concise manner:\n\n"+text))
	if err != nil {
		return "", err
	}
	return formatResponse(resp), nil
}

func (s *GeminiService) Paraphrase(ctx context.Context, text string) (string, error) {
	model := s.client.GenerativeModel(s.model)
	resp, err := model.GenerateContent(ctx, genai.Text("Paraphrase the following note making it clearer:\n\n"+text))
	if err != nil {
		return "", err
	}
	return formatResponse(resp), nil
}

func (s *GeminiService) GenerateMindmap(ctx context.Context, text string) (string, error) {
	model := s.client.GenerativeModel(s.model)
	resp, err := model.GenerateContent(ctx, genai.Text("Create a Mermaid.js mindmap syntax for the following text. Only return the Mermaid syntax starting with 'mindmap' and nothing else. Do not use markdown code blocks. Ensure the syntax is valid for Mermaid.js.\n\n"+text))
	if err != nil {
		return "", err
	}
	return formatResponse(resp), nil
}

func formatResponse(resp *genai.GenerateContentResponse) string {
	var result string
	for _, cand := range resp.Candidates {
		if cand.Content != nil {
			for _, part := range cand.Content.Parts {
				result += fmt.Sprintf("%v", part)
			}
		}
	}
	return result
}

func GetAIService(ctx context.Context, settings models.Settings) (AIService, error) {
	switch settings.Provider {
	case "google":
		return NewGeminiService(ctx, settings.Model)
	// Add other providers here (Ollama, Groq)
	default:
		return NewGeminiService(ctx, "gemini-2.0-flash")
	}
}
