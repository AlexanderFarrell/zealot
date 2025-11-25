package main

import (
	_ "embed"
	"zealotd/apps/account"
	"zealotd/apps/item"
	"zealotd/web"
)

//go:embed init.psql
var initSQL string



func main() {
	web.DatabaseStart()

	web.InitDatabaseIfEmpty(initSQL)

	app := web.InitServer()

	account.InitRouter(app)
	item.InitRouter(app)

	web.RunServer(app)
}