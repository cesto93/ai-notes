package main

import (
	"log"
	"net/http"
	"os"
	"path/filepath"
	"strings"

	"github.com/gin-contrib/cors"
	"github.com/gin-gonic/gin"
	"github.com/pier/ai-notes/backend-go/internal/ai"
	"github.com/pier/ai-notes/backend-go/internal/config"
	"github.com/pier/ai-notes/backend-go/internal/models"
	"github.com/pier/ai-notes/backend-go/internal/storage"
	"github.com/pier/ai-notes/backend-go/internal/utils"
)

func main() {
	config.LoadConfig()

	r := gin.Default()

	// CORS configuration
	r.Use(cors.New(cors.Config{
		AllowOrigins:     []string{"*"},
		AllowMethods:     []string{"GET", "POST", "PUT", "DELETE", "OPTIONS"},
		AllowHeaders:     []string{"Origin", "Content-Type", "Accept"},
		AllowCredentials: true,
	}))

	// API Routes
	api := r.Group("/")
	{
		api.GET("/notes", func(c *gin.Context) {
			notes, err := storage.ListNotes()
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			c.JSON(http.StatusOK, notes)
		})

		api.GET("/note/*path", func(c *gin.Context) {
			path := c.Param("path")
			path = strings.TrimPrefix(path, "/")
			note, err := storage.GetNote(path)
			if err != nil {
				c.JSON(http.StatusNotFound, gin.H{"error": "Note not found"})
				return
			}
			c.JSON(http.StatusOK, note)
		})

		api.POST("/note", func(c *gin.Context) {
			var req models.NoteRequest
			if err := c.ShouldBindJSON(&req); err != nil {
				c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
				return
			}
			err := storage.SaveNote(req.Content, req.Title, req.Directory)
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			c.JSON(http.StatusOK, gin.H{"message": "Note saved"})
		})

		api.PUT("/note", func(c *gin.Context) {
			var req models.UpdateNoteRequest
			if err := c.ShouldBindJSON(&req); err != nil {
				c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
				return
			}
			err := storage.UpdateNote(req)
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			c.JSON(http.StatusOK, gin.H{"message": "Note updated"})
		})

		api.POST("/note/move", func(c *gin.Context) {
			var req models.MoveNoteRequest
			if err := c.ShouldBindJSON(&req); err != nil {
				c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
				return
			}
			err := storage.MoveNote(req)
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			c.JSON(http.StatusOK, gin.H{"message": "Note moved"})
		})

		api.POST("/directory", func(c *gin.Context) {
			var req models.DirectoryRequest
			if err := c.ShouldBindJSON(&req); err != nil {
				c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
				return
			}
			err := storage.CreateDirectory(req.Name)
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			c.JSON(http.StatusOK, gin.H{"message": "Directory created"})
		})

		// Single wildcard route handles both /note/title and /note/directory/title
		api.DELETE("/note/*path", func(c *gin.Context) {
			path := strings.TrimPrefix(c.Param("path"), "/")
			parts := strings.SplitN(path, "/", 2)
			var directory, title string
			if len(parts) == 2 {
				directory = parts[0]
				title = parts[1]
			} else {
				directory = ""
				title = parts[0]
			}
			if directory == "none" {
				directory = ""
			}
			err := storage.DeleteNote(title, directory)
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			c.JSON(http.StatusOK, gin.H{"message": "Note deleted"})
		})

		api.DELETE("/directory/:name", func(c *gin.Context) {
			name := c.Param("name")
			err := storage.DeleteDirectory(name)
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			c.JSON(http.StatusOK, gin.H{"message": "Directory deleted"})
		})

		api.GET("/settings", func(c *gin.Context) {
			settings, err := storage.GetSettings()
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			c.JSON(http.StatusOK, settings)
		})

		api.POST("/settings", func(c *gin.Context) {
			var req models.SettingsRequest
			if err := c.ShouldBindJSON(&req); err != nil {
				c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
				return
			}
			err := storage.SaveSettings(req.Provider, req.Model)
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			c.JSON(http.StatusOK, gin.H{"message": "Settings saved"})
		})

		api.POST("/summarize", func(c *gin.Context) {
			var req models.SummarizeRequest
			if err := c.ShouldBindJSON(&req); err != nil {
				c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
				return
			}
			settings, _ := storage.GetSettings()
			service, err := ai.GetAIService(c.Request.Context(), settings)
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			result, err := service.Summarize(c.Request.Context(), req.Text)
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			c.JSON(http.StatusOK, gin.H{"result": result})
		})

		api.POST("/paraphrase", func(c *gin.Context) {
			var req models.SummarizeRequest
			if err := c.ShouldBindJSON(&req); err != nil {
				c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
				return
			}
			settings, _ := storage.GetSettings()
			service, err := ai.GetAIService(c.Request.Context(), settings)
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			result, err := service.Paraphrase(c.Request.Context(), req.Text)
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			c.JSON(http.StatusOK, gin.H{"result": result})
		})

		api.POST("/mindmap", func(c *gin.Context) {
			var req models.SummarizeRequest
			if err := c.ShouldBindJSON(&req); err != nil {
				c.JSON(http.StatusBadRequest, gin.H{"error": err.Error()})
				return
			}
			settings, _ := storage.GetSettings()
			service, err := ai.GetAIService(c.Request.Context(), settings)
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			result, err := service.GenerateMindmap(c.Request.Context(), req.Text)
			if err != nil {
				c.JSON(http.StatusInternalServerError, gin.H{"error": err.Error()})
				return
			}
			c.JSON(http.StatusOK, gin.H{"result": result})
		})
	}

	// Serve static files
	staticDir := utils.GetPath("static")
	if _, err := os.Stat(staticDir); err == nil {
		r.Static("/_app", filepath.Join(staticDir, "_app"))
		r.NoRoute(func(c *gin.Context) {
			path := filepath.Join(staticDir, c.Request.URL.Path)
			if _, err := os.Stat(path); err == nil && !strings.HasSuffix(path, "/") {
				c.File(path)
				return
			}
			c.File(filepath.Join(staticDir, "index.html"))
		})
	}

	port := utils.GetEnv("PORT", "8000")
	log.Printf("Server starting on port %s", port)
	r.Run(":" + port)
}
