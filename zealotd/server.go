package main

import (
	"zealotd/apps/item"
	"zealotd/web"
)

func main() {
	web.DatabaseStart()
	app := web.InitServer()

	item.InitRouter(app)

	web.RunServer(app)
}