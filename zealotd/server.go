package main

import (
	"context"
	_ "embed"
	"fmt"
	"time"
	"zealotd/apps/account"
	"zealotd/apps/item"
	"zealotd/web"
)

//go:embed init.psql
var initSQL string

func initSchema() {
	// Check if a table exists, if not, need to initialize.
	var table *string
	err := web.Database.QueryRow(`SELECT to_regclass('public.item')`).Scan(&table)
	if err != nil {
		panic(fmt.Errorf("error determining if database is initialized, stopping: %v", err))
	}

	if table == nil {
		fmt.Printf("Database Init - Started.\n")

		// Not initialized. Require 10 seconds max to initialize.
		ctx, cancel := context.WithTimeout(context.Background(), 10 * time.Second)
		defer cancel()

		tx, err := web.Database.BeginTx(ctx, nil)
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

func main() {
	web.DatabaseStart()

	initSchema()

	app := web.InitServer()

	account.InitRouter(app)
	item.InitRouter(app)

	web.RunServer(app)
}