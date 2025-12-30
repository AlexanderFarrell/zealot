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
				var f FileStat = FileStat{
					Path:     file.Name(),
					Size:     0,
					IsFolder: file.IsDir(),
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

	// Delete file
	api.Delete("/*", func (c * fiber.Ctx) error {
		return c.SendStatus(fiber.StatusNotImplemented)
	})

	return api
}
