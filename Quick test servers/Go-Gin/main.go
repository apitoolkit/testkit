package main

import (
	"fmt"

	"github.com/gin-gonic/gin"
)

type Todo struct {
	ID        string `json:"_id"`
	Task      any    `json:"task"`
	Completed bool   `json:"completed"`
}

var todos = []Todo{
	{ID: "1", Task: "First task", Completed: false},
	{ID: "2", Task: "Second task", Completed: true},
	{ID: "3", Task: "Third task", Completed: false},
}

func main() {
	r := gin.Default()

	r.GET("/todos", func(c *gin.Context) {
		c.JSON(200, gin.H{"todos": todos})
	})

	r.DELETE("/todos/:id", func(c *gin.Context) {
		id := c.Param("id")
		for i, todo := range todos {
			if todo.ID == id {
				todos = append(todos[:i], todos[i+1:]...)
				break
			}
		}
		c.JSON(200, gin.H{"message": "Todo deleted!"})
	})

	r.POST("/todos", func(c *gin.Context) {
		var json Todo
		if err := c.ShouldBindJSON(&json); err != nil {
			c.JSON(400, gin.H{"error": err.Error()})
			return
		}
		fmt.Println(json)
		newTodo := Todo{ID: string(len(todos) + 1), Task: json.Task, Completed: false}
		todos = append(todos, newTodo)
		c.JSON(201, newTodo)
	})

	r.Run(":3000")
}
