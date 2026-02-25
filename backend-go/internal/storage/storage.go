package storage

import (
	"encoding/json"
	"fmt"
	"os"
	"path/filepath"
	"strings"
	"sync"

	"github.com/pier/ai-notes/backend-go/internal/models"
	"github.com/pier/ai-notes/backend-go/internal/utils"
)

func getNotesDir() string {
	return utils.GetPath(utils.GetEnv("NOTES_DIR", "notes"))
}

func getDBFile() string {
	return utils.GetPath(utils.GetEnv("DB_FILE", "notes_db.json"))
}

var (
	mu sync.Mutex
)

// tinyDBRecord represents a single record stored by TinyDB (string-keyed map).
type tinyDBRecord map[string]json.RawMessage

// tinyDBFile mirrors the on-disk TinyDB JSON format:
//
//	{ "_default": { "1": {...}, "2": {...} }, "settings": { "1": {...} } }
type tinyDBFile map[string]tinyDBRecord

// DB is our in-memory representation after parsing.
type DB struct {
	Notes    []models.NoteMetadata
	Settings models.Settings
}

func readDB() (*DB, error) {
	dbFile := getDBFile()
	if _, err := os.Stat(dbFile); os.IsNotExist(err) {
		return &DB{
			Notes:    []models.NoteMetadata{},
			Settings: models.Settings{Provider: "google", Model: "gemini-2.0-flash"},
		}, nil
	}

	data, err := os.ReadFile(dbFile)
	if err != nil {
		return nil, err
	}

	var raw tinyDBFile
	if err := json.Unmarshal(data, &raw); err != nil {
		return nil, fmt.Errorf("failed to parse db file: %w", err)
	}

	db := &DB{
		Notes:    []models.NoteMetadata{},
		Settings: models.Settings{Provider: "google", Model: "gemini-2.0-flash"},
	}

	// Parse notes from "_default" table
	if defaultTable, ok := raw["_default"]; ok {
		for _, v := range defaultTable {
			var note models.NoteMetadata
			if err := json.Unmarshal(v, &note); err == nil && note.Argument != "" {
				db.Notes = append(db.Notes, note)
			}
		}
	}

	// Parse settings from "settings" table
	if settingsTable, ok := raw["settings"]; ok {
		for _, v := range settingsTable {
			var s models.Settings
			if err := json.Unmarshal(v, &s); err == nil && s.Provider != "" {
				db.Settings = s
				break
			}
		}
	}

	return db, nil
}

func writeDB(db *DB) error {
	dbFile := getDBFile()

	// Read the existing raw file so we preserve any extra tables.
	raw := make(tinyDBFile)
	if data, err := os.ReadFile(dbFile); err == nil {
		_ = json.Unmarshal(data, &raw)
	}

	// Rebuild the "_default" table as a string-keyed map.
	defaultTable := make(tinyDBRecord)
	for i, note := range db.Notes {
		key := fmt.Sprintf("%d", i+1)
		b, err := json.Marshal(note)
		if err != nil {
			return err
		}
		defaultTable[key] = b
	}
	raw["_default"] = defaultTable

	// Rebuild the "settings" table.
	settingsTable := make(tinyDBRecord)
	b, err := json.Marshal(db.Settings)
	if err != nil {
		return err
	}
	settingsTable["1"] = b
	raw["settings"] = settingsTable

	out, err := json.MarshalIndent(raw, "", "  ")
	if err != nil {
		return err
	}
	return os.WriteFile(dbFile, out, 0644)
}

func CreateDirectory(directory string) error {
	path := filepath.Join(getNotesDir(), directory)
	return os.MkdirAll(path, 0755)
}

func SaveNote(content, title, directory string) error {
	mu.Lock()
	defer mu.Unlock()

	targetDir := getNotesDir()
	if directory != "" {
		targetDir = filepath.Join(getNotesDir(), directory)
	}
	if err := os.MkdirAll(targetDir, 0755); err != nil {
		return err
	}

	filename := strings.ReplaceAll(title, " ", "_") + ".md"
	notePath := filepath.Join(targetDir, filename)
	if err := os.WriteFile(notePath, []byte(content), 0644); err != nil {
		return err
	}

	db, err := readDB()
	if err != nil {
		return err
	}

	db.Notes = append(db.Notes, models.NoteMetadata{Argument: title, Directory: directory})
	return writeDB(db)
}

func GetSettings() (models.Settings, error) {
	db, err := readDB()
	if err != nil {
		return models.Settings{}, err
	}
	if db.Settings.Provider == "" {
		return models.Settings{Provider: "google", Model: "gemini-2.0-flash"}, nil
	}
	return db.Settings, nil
}

func SaveSettings(provider, model string) error {
	mu.Lock()
	defer mu.Unlock()

	db, err := readDB()
	if err != nil {
		return err
	}

	db.Settings = models.Settings{Provider: provider, Model: model}
	return writeDB(db)
}

func ListNotes() (map[string]interface{}, error) {
	notesDir := getNotesDir()
	if _, err := os.Stat(notesDir); os.IsNotExist(err) {
		return map[string]interface{}{"files": []string{}, "directories": map[string][]string{}}, nil
	}

	files := []string{}
	directories := make(map[string][]string)

	items, err := os.ReadDir(notesDir)
	if err != nil {
		return nil, err
	}

	for _, item := range items {
		if item.IsDir() {
			if strings.HasPrefix(item.Name(), ".") {
				continue
			}
			subdirPath := filepath.Join(notesDir, item.Name())
			subItems, _ := os.ReadDir(subdirPath)
			notes := []string{}
			for _, subItem := range subItems {
				if !subItem.IsDir() && strings.HasSuffix(subItem.Name(), ".md") {
					notes = append(notes, subItem.Name())
				}
			}
			directories[item.Name()] = notes
		} else {
			if strings.HasSuffix(item.Name(), ".md") {
				files = append(files, item.Name())
			}
		}
	}

	return map[string]interface{}{
		"files":       files,
		"directories": directories,
	}, nil
}

func GetNote(path string) (map[string]interface{}, error) {
	fullPath := filepath.Join(getNotesDir(), path)
	if _, err := os.Stat(fullPath); os.IsNotExist(err) {
		return nil, fmt.Errorf("note not found")
	}

	content, err := os.ReadFile(fullPath)
	if err != nil {
		return nil, err
	}

	parts := strings.Split(path, string(os.PathSeparator))
	var directory, filename string
	if len(parts) > 1 {
		directory = parts[0]
		filename = parts[1]
	} else {
		directory = ""
		filename = parts[0]
	}

	filenameNoExt := strings.TrimSuffix(filename, ".md")
	metadata, _ := GetNoteMetadata(filenameNoExt, directory)

	title := filenameNoExt
	if metadata != nil {
		title = metadata.Argument
	}

	return map[string]interface{}{
		"content":   string(content),
		"title":     title,
		"directory": directory,
	}, nil
}

func GetNoteMetadata(filenameNoExt, directory string) (*models.NoteMetadata, error) {
	db, err := readDB()
	if err != nil {
		return nil, err
	}

	for _, n := range db.Notes {
		if strings.ReplaceAll(n.Argument, " ", "_") == filenameNoExt && n.Directory == directory {
			return &n, nil
		}
	}
	return nil, nil
}

func UpdateNote(req models.UpdateNoteRequest) error {
	mu.Lock()
	defer mu.Unlock()

	oldFilename := strings.ReplaceAll(req.OldTitle, " ", "_") + ".md"
	oldFilepath := filepath.Join(getNotesDir(), req.OldDirectory, oldFilename)

	newTargetDir := getNotesDir()
	if req.NewDirectory != "" {
		newTargetDir = filepath.Join(getNotesDir(), req.NewDirectory)
	}
	os.MkdirAll(newTargetDir, 0755)

	newFilename := strings.ReplaceAll(req.NewTitle, " ", "_") + ".md"
	newFilepath := filepath.Join(newTargetDir, newFilename)

	if oldFilepath != newFilepath {
		if _, err := os.Stat(oldFilepath); err == nil {
			os.Remove(oldFilepath)
		}
	}

	if err := os.WriteFile(newFilepath, []byte(req.NewContent), 0644); err != nil {
		return err
	}

	db, err := readDB()
	if err != nil {
		return err
	}

	for i, n := range db.Notes {
		if n.Argument == req.OldTitle && n.Directory == req.OldDirectory {
			db.Notes[i] = models.NoteMetadata{Argument: req.NewTitle, Directory: req.NewDirectory}
			break
		}
	}
	return writeDB(db)
}

func MoveNote(req models.MoveNoteRequest) error {
	mu.Lock()
	defer mu.Unlock()

	filenameNoExt := strings.ReplaceAll(req.Title, " ", "_")
	// Read DB without lock (we already hold it)
	db, err := readDB()
	if err != nil {
		return err
	}

	actualTitle := req.Title
	for _, n := range db.Notes {
		if strings.ReplaceAll(n.Argument, " ", "_") == filenameNoExt && n.Directory == req.OldDirectory {
			actualTitle = n.Argument
			break
		}
	}

	for i, n := range db.Notes {
		if n.Argument == actualTitle && n.Directory == req.OldDirectory {
			db.Notes[i].Directory = req.NewDirectory
			break
		}
	}
	if err = writeDB(db); err != nil {
		return err
	}

	oldFilename := filenameNoExt + ".md"
	oldFilepath := filepath.Join(getNotesDir(), req.OldDirectory, oldFilename)

	newTargetDir := getNotesDir()
	if req.NewDirectory != "" {
		newTargetDir = filepath.Join(getNotesDir(), req.NewDirectory)
	}
	os.MkdirAll(newTargetDir, 0755)

	newFilepath := filepath.Join(newTargetDir, oldFilename)

	if _, err := os.Stat(oldFilepath); err == nil {
		return os.Rename(oldFilepath, newFilepath)
	}
	return nil
}

func DeleteNote(title, directory string) error {
	mu.Lock()
	defer mu.Unlock()

	filenameNoExt := strings.ReplaceAll(title, " ", "_")
	filename := filenameNoExt + ".md"
	path := filepath.Join(getNotesDir(), directory, filename)

	if _, err := os.Stat(path); err == nil {
		os.Remove(path)
	}

	db, err := readDB()
	if err != nil {
		return err
	}

	newNotes := []models.NoteMetadata{}
	for _, n := range db.Notes {
		if strings.ReplaceAll(n.Argument, " ", "_") == filenameNoExt && n.Directory == directory {
			continue
		}
		if n.Argument == title && n.Directory == directory {
			continue
		}
		newNotes = append(newNotes, n)
	}
	db.Notes = newNotes
	return writeDB(db)
}

func DeleteDirectory(directory string) error {
	mu.Lock()
	defer mu.Unlock()

	path := filepath.Join(getNotesDir(), directory)
	if _, err := os.Stat(path); err == nil {
		os.RemoveAll(path)
	}

	db, err := readDB()
	if err != nil {
		return err
	}

	newNotes := []models.NoteMetadata{}
	for _, n := range db.Notes {
		if n.Directory == directory {
			continue
		}
		newNotes = append(newNotes, n)
	}
	db.Notes = newNotes
	return writeDB(db)
}
