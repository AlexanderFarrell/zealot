package web

import (
	"context"
	"database/sql"
	"errors"
	"fmt"
	_ "github.com/lib/pq"
	"os"
	"strconv"
	"time"
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
	username := GetEnvVar("DB_USERNAME", DbUserDefault)
	host := GetEnvVar("DB_HOSTNAME", DbHostDefault)
	port := GetEnvVar("DB_PORT", DbPortDefault)
	database := GetEnvVar("DB_DATABASE", DbDatabaseDefault)
	sslmode := GetEnvVar("DB_SSL_MODE", DbSSLModeDefault)

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

func UpdateRow(account_id int, tableName string, identifier string, identifierRowName string,
	fields map[string]interface{}, allowedFields map[string]int) error {
	query := "UPDATE " + tableName + " SET "
	args := []interface{}{}
	i := 1

	for field, value := range fields {
		if _, ok := allowedFields[field]; ok {
			query += fmt.Sprintf(" %s = $%v, ", field, i)
			args = append(args, value)
			i++
		}
	}

	query = query[:len(query)-2] +
		" where " + identifierRowName + " = $" + strconv.Itoa(i) +
		" and account_id = $" + strconv.Itoa(i+1)

	args = append(args, identifier)
	args = append(args, account_id)

	_, err := Database.Exec(query, args...)
	return err
}

func DeleteRow(accountID int, tableName string, identifier int, identifierRowName string) error {
	query := "DELETE FROM " + tableName + " WHERE " + identifierRowName + "=$1 " + " AND account_id=$2"
	_, err := Database.Exec(query, identifier, accountID)
	return err
}

func InitDatabaseIfEmpty(initSQL string) {
	// Check if a table exists, if not, need to initialize.
	var table *string
	err := Database.QueryRow(`SELECT to_regclass('public.item')`).Scan(&table)
	if err != nil {
		panic(fmt.Errorf("error determining if database is initialized, stopping: %v", err))
	}

	if table == nil {
		fmt.Printf("Database Init - Started.\n")

		// Not initialized. Require 10 seconds max to initialize.
		ctx, cancel := context.WithTimeout(context.Background(), 10*time.Second)
		defer cancel()

		tx, err := Database.BeginTx(ctx, nil)
		if err != nil {
			panic(fmt.Errorf("begin tx: %w", err))
		}

		if _, err := tx.ExecContext(ctx, initSQL); err != nil {
			_ = tx.Rollback()
			panic(fmt.Errorf("begin tx: %w", err))
		}

		if err := tx.Commit(); err != nil {
			panic(fmt.Errorf("begin tx: %w", err))
		}

		fmt.Printf("Database Init - Complete\n")
	} else {
		fmt.Printf("Database already initialized\n")
	}
}
