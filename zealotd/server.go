package main

import (
	_ "embed"
	"zealotd/apps/account"
	"zealotd/apps/item"
	"zealotd/apps/settings"
	"zealotd/web"
)

//go:embed init.psql
var initSQL string



func main() {
	web.DatabaseStart()

	web.InitDatabaseIfEmpty(initSQL)

	app := web.InitServer()

	account.InitRouter(app, settings.SettingsHandler)
	item.InitRouter(app)

	web.RunServer(app)
}