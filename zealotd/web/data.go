package web

import (
	"database/sql"
	"errors"
	"fmt"
	_ "github.com/lib/pq"
	"os"
)

const (
	DbUserDefault     = "postgres"
	DbPassDefault     = ""
	DbHostDefault     = "localhost"
	DbPortDefault     = "5432"
	DbDatabaseDefault = "postgres"
	DbSSLModeDefault  = "disable"
)

var (
	Database *sql.DB
)

func DatabaseStart() {
	uri := getDatabaseURI()
	db, err := sql.Open("postgres", uri)
	if err != nil {
		fmt.Printf("URI Failed: %s \n", uri)
		fmt.Printf("Error: %s \n", err.Error())
		panic("Failed to connect to database")
	}
	Database = db
}

func getDatabaseURI() string {
	//dbUri := os.Getenv("DATABASE_URL")
	//if dbUri == "" {
	//	return dbUri
	//}

	username := os.Getenv("DB_USERNAME")
	if username == "" {
		username = DbUserDefault
	}

	//password := os.Getenv("DB_PASSWORD")
	//if password == "" {
	//	password = DbPassDefault
	//}

	host := os.Getenv("DB_HOSTNAME")
	if host == "" {
		host = DbHostDefault
	}

	port := os.Getenv("DB_PORT")
	if port == "" {
		port = DbPortDefault
	}

	database := os.Getenv("DB_DATABASE")
	if database == "" {
		database = DbDatabaseDefault
	}

	sslmode := os.Getenv("DB_SSL_MODE")
	if sslmode == "" {
		sslmode = DbSSLModeDefault
	}

	uri := fmt.Sprintf("host=%s port=%s user=%s dbname=%s sslmode=%s",
		host, port, username, database, sslmode)

	// Add SSL Settings
	sslRootCert := os.Getenv("DB_SSL_Root_Cert")
	if sslRootCert != "" {
		uri += fmt.Sprintf(" sslrootcert=%s", sslRootCert)
	}

	sslClientCert := os.Getenv("")
	if sslClientCert != "" {
		uri += fmt.Sprintf(" sslcert=%s", sslClientCert)
	}

	sslClientKey := os.Getenv("DB_SSL_Client_Key")
	if sslClientKey != "" {
		uri += fmt.Sprintf(" sslkey=%s", sslClientKey)
	}

	password := os.Getenv("DB_PASSWORD")
	if password != "" {
		uri += fmt.Sprintf(" password=%s", password)
	}

	return uri
}

func CheckResultForChanges(result sql.Result, err error) error {
	if err == nil {
		rows, err := result.RowsAffected()
		if err != nil {
			return err
		}
		if rows == 0 {
			return errors.New("did not find goal")
		}
	}

	return err
}
