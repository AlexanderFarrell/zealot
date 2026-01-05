package media

import (
	"zealotd/apps/account"
	"os"
	"fmt"
	"zealotd/web"
	"github.com/gofiber/fiber/v2"
	"net/url"
	"path/filepath"
)

type FileStat struct {
	Path string `json:"path"`
	Size int64 `json:"size"`
	IsFolder bool `json:"is_folder"`
	Modified int64 `json:"modified_at"`
}

func InitRouter(app *fiber.App) fiber.Router {
	InitEnvVariables()

	api := app.Group("/media")
	api.Use(account.RequireLoginMiddleware)

	// Get media
	api.Get("/*", func (c * fiber.Ctx) error {
		username := web.GetKeyFromSession(c, "username")

		basePath, err := UserPath(username, "")
		if err != nil {
			fmt.Printf("%v", err)
			return c.SendStatus(400)
		}

		// Create a folder for the user if it doesn't exist
		err = os.MkdirAll(basePath, os.ModePerm)
		if err != nil {
			fmt.Printf("%v", err)
			return c.SendStatus(400)
		}

		pathName := c.Params("*")
		pathName, err = url.QueryUnescape(c.Params("*"))
		if err != nil {
			fmt.Printf("%v\n", err)
			return c.SendStatus(400)
		}

		path, err := UserPath(username, pathName)
		if err != nil {
			fmt.Printf("%v\n", err)
			return c.SendStatus(400)
		}
		s, err := os.Stat(path)
		if err != nil {
			fmt.Printf("%v\n", err)
			if os.IsNotExist(err) {
				return c.SendStatus(404)
			}
			return c.SendStatus(500)
		}

		if s.IsDir() {
			// Get all the fileStats from the accounts folder
			files, err := os.ReadDir(path)
			if err != nil {
				fmt.Printf("%v", err)
				return c.SendStatus(400)
			}
			var fileStats []FileStat
			for _, file := range files {
				info, err := file.Info()
				if err != nil {
					fmt.Printf("Error getting file info: %v\n", err)
					continue
				}
				var f FileStat = FileStat{
					Path:     file.Name(),
					Size:     info.Size(),
					IsFolder: file.IsDir(),
					Modified: info.ModTime().Unix(),
				}
				fileStats = append(fileStats, f)
			}

			return c.JSON(fiber.Map{
				"files": fileStats,
			})
		} else {
			etag, err := CalculateEtag(path)
			if err != nil {
				fmt.Printf("%v\n", err)
				return c.SendStatus(500)
			}

			if c.Get("If-None-Match") == etag {
				return c.SendStatus(304)
			}

			c.Set("ETag", etag)

			// Serve the file
			return c.SendFile(path)
		}
	})

	// Make folder
	api.Post("/mkdir", func (c *fiber.Ctx) error {
		username := web.GetKeyFromSession(c, "username")

		payload := struct {
			Folder string `json:"folder"`
		}{}
		err := c.BodyParser(&payload)
		if err != nil {
			fmt.Printf("%v", err)
			return c.SendStatus(400)
		}

		path, err := UserPath(username, payload.Folder)
		if err != nil {
			fmt.Printf("%v", err)
			return c.SendStatus(400)
		}
		if len(path) < 1024 {
			err = os.MkdirAll(path, os.ModePerm)
			if err != nil {
				fmt.Printf("%v", err)
				return c.SendStatus(500)
			} else {
				return c.SendStatus(200)
			}
		} else {
			return c.Status(400).SendString("Path too large")
		}
	}) 

	// Upload file
	api.Post("/*", func (c * fiber.Ctx) error {
		username := web.GetKeyFromSession(c, "username")

		file, err := c.FormFile("file")
		if err != nil {
			fmt.Printf("%v", err)
			return c.SendStatus(400)
		}

		filename := filepath.Base(file.Filename)
		if filename == "." || filename == string(os.PathSeparator) || filename == "" {
			return c.SendStatus(400)
		}

		pathName := c.Params("*")
		pathName, err = url.QueryUnescape(c.Params("*"))
		if err != nil {
			fmt.Printf("%v\n", err)
			return c.SendStatus(400)
		}

		dirPath, err := UserPath(username, pathName)
		if err != nil {
			fmt.Printf("%v", err)
			return c.SendStatus(400)
		}

		// Create a folder for the user if it doesn't exist
		err = os.MkdirAll(dirPath, os.ModePerm)
		if err != nil {
			fmt.Printf("%v", err)
			return c.SendStatus(400)
		}

		// Save the file to the local filesystem in the folder for the account
		filePath, err := UserPath(username, filepath.Join(pathName, filename))
		if err != nil {
			fmt.Printf("%v", err)
			return c.SendStatus(400)
		}
		err = c.SaveFile(file, filePath)
		if err != nil {
			fmt.Printf("%v", err)
			return c.SendStatus(400)
		}

		return c.SendStatus(200)
	})

	api.Patch("/rename", func (c * fiber.Ctx) error {
		username := web.GetKeyFromSession(c, "username")

		payload := struct {
			OldLocation string `json:"old_location"`
			NewName string `json:"new_name"`
		}{}

		if err := c.BodyParser(&payload); err != nil {
			fmt.Printf("%v\n", err)
			return c.SendStatus(fiber.StatusBadRequest)
		}

		if payload.OldLocation == "" || payload.NewName == "" {
			return c.SendStatus(fiber.StatusBadRequest)
		}

		// Resolve
		oldPath, err := UserPath(username, payload.OldLocation)
		if err != nil {
			fmt.Printf("%v\n", err)
			return c.SendStatus(fiber.StatusBadRequest)
		}

		// Keep rename in same folder
		dir := filepath.Dir(payload.OldLocation)
		cleanName := filepath.Base(payload.NewName)
		if cleanName == "." || cleanName == string(os.PathSeparator) || cleanName == "" {
			return c.SendStatus(fiber.StatusBadRequest)
		}

		newRel := filepath.Join(dir, cleanName)
		newPath, err := UserPath(username, newRel)

		if err != nil {
			fmt.Printf("%v\n", err)
			return c.SendStatus(fiber.StatusBadRequest)
		}

		if err := os.Rename(oldPath, newPath); err != nil {
			fmt.Printf("%v\n", err)
			return c.SendStatus(fiber.StatusInternalServerError)
		}

		return c.SendStatus(fiber.StatusOK)
	})

	// Delete file
	api.Delete("/*", func (c * fiber.Ctx) error {
		username := web.GetKeyFromSession(c, "username")

		pathName := c.Params("*")
		pathName, err := url.QueryUnescape(pathName)
		if err != nil {
			fmt.Printf("%v\n", err)
			return c.SendStatus(fiber.StatusBadRequest)
		}

		if pathName == "" {
			// If this happens, it will delete user root
			return c.SendStatus(fiber.StatusBadRequest)
		}

		fullPath, err := UserPath(username, pathName)
		if err != nil {
			fmt.Printf("%v\n", err)
			return c.SendStatus(fiber.StatusBadRequest)
		}

		if err := os.RemoveAll(fullPath); err != nil {
			fmt.Printf("%v\n", err)
			return c.SendStatus(fiber.StatusInternalServerError)
		}

		return c.SendStatus(fiber.StatusOK)
	})

	return api
}
