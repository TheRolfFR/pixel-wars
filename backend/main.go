// main package of NIxel-Wars backend
package main

import (
	"fmt"
  
	"niaefeup/backend-nixel-wars/controller"
	"niaefeup/backend-nixel-wars/model"
	"niaefeup/backend-nixel-wars/web"

	"github.com/gin-gonic/gin"
)

func main() {
	r := gin.Default()

	config := model.LoadConfigurationFile()
	controller.RedisCreateBitFieldIfNotExists(&config)
	/*
		Add your groups here...
	*/
	api.AddRoutes(r)

	r.Static("/assets", "../frontend/dist/assets")
	r.StaticFile("/vite.svg", "../frontend/dist/vite.svg")
	r.StaticFile("/", "../frontend/dist/index.html")

	//TODO: serve this as HTTPS
	if err := r.Run(":8080"); err != nil {
		fmt.Println("Failed to start server...")
		fmt.Println(err.Error())
	}
}
