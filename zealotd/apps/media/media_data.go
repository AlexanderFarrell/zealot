package media

import (
	"fmt"
	"os"
	"strings"
	"zealotd/web"
	"encoding/hex"
	"crypto/sha1"
	"io"
	"path/filepath"
)

const (
	defaultUploadFolder = "./uploads"
)

var (
	uploadFolder string
)

func InitEnvVariables() {

	uploadFolder = web.GetEnvVar("UPLOAD_FOLDER", defaultUploadFolder)

	if !CheckIfExists(uploadFolder) && uploadFolder != defaultUploadFolder {
		panic("Set upload folder: " + uploadFolder + " does not exist")
	}
}

func DeleteFolder(username string, path string) error {
	path = SanitizedForFileIO(path)
	username = SanitizedForFileIO(username)
	p := fmt.Sprintf(uploadFolder+"/%s/%s", username, path)
	err := os.RemoveAll(p)
	return err
}

func SanitizedForFileIO(s string) string {
	s = strings.Replace(s, "..", "", -1)
	//s = strings.Replace(s, "/", "", -1)
	//s = strings.Replace(s, "\\", "", -1)
	return s
}

func UserPath(username string, rel string) (string, error) {
	username = SanitizedForFileIO(username)
	rel = strings.TrimSpace(rel)

	if strings.Contains(rel, "\x00") {
		return "", fmt.Errorf("invalid path")
	}

	cleanRel := filepath.Clean("/" + rel)
	cleanRel = strings.TrimPrefix(cleanRel, string(os.PathSeparator))

	base := filepath.Clean(filepath.Join(uploadFolder, username))
	full := filepath.Clean(filepath.Join(base, cleanRel))

	if full != base && !strings.HasPrefix(full, base+string(os.PathSeparator)) {
		return "", fmt.Errorf("invalid path")
	}

	return full, nil
}

func CheckIfExists(path string) bool {
	_, err := os.Stat(path)
	if err == nil {
		return true
	}
	if os.IsNotExist(err) {
		return false
	}

	return false
}

func CalculateEtag(filePath string) (string, error) {
	file, err := os.Open(filePath)
	if err != nil {
		return "", err
	}
	defer file.Close()

	hash := sha1.New()
	if _, err := io.Copy(hash, file); err != nil {
		return "", err
	}

	return hex.EncodeToString(hash.Sum(nil)), nil
}

func GetAt() {
	
}
