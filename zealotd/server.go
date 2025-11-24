package main

import (
	"zealotd/apps/account"
	"zealotd/apps/item"
	"zealotd/web"
)

func main() {
	web.DatabaseStart()
	app := web.InitServer()

	account.InitRouter(app)
	item.InitRouter(app)

	web.RunServer(app)
}