package main

import (
	_ "embed"
	"zealotd/apps/account"
	"zealotd/apps/item"
	"zealotd/apps/media"
	"zealotd/apps/planner"
	"zealotd/apps/settings"
	"zealotd/web"
)

//go:embed init.psql
var initSQL string

func main() {
	web.DatabaseStart()
	web.InitDatabaseIfEmpty(initSQL)
	web.InitSessions()

	app := web.InitServer()

	account.InitRouter(app, settings.SettingsHandler)
	item.InitRouter(app)
	planner.InitRouter(app)
	media.InitRouter(app)

	web.RunServer(app)
}
