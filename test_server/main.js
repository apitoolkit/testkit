const express = require('express');
const bodyParser = require('body-parser');

// Create Express application
const app = express();
app.use(bodyParser.json());

// Array to store todo items (acting as a simple in-memory database)
let todos = [];

// Get all todos
app.get('/todos', (req, res) => {
  res.json({ "tasks": ["task one", 4, "task two", "task three"], "empty_str": "", "empty_arr": [], resp_null: null });
});

// Get a specific todo by ID
app.get('/todos/:id', (req, res) => {
  const id = parseInt(req.params.id);
  const todo = todos.find(todo => todo.id === id);

  if (todo) {
    res.json(todo);
  } else {
    res.status(404).json({ error: 'Todo not found' });
  }
});

// Create a new todo
app.post('/todos', (req, res) => {
  const newTodo = req.body;
  newTodo.id = todos.length + 1;
  todos.push(newTodo);
  res.status(201).json(newTodo);
});

// Update a todo
app.put('/todos/:id', (req, res) => {
  const id = parseInt(req.params.id);
  const updatedTodo = req.body;
  const index = todos.findIndex(todo => todo.id === id);

  if (index !== -1) {
    todos[index] = { ...todos[index], ...updatedTodo };
    res.json(todos[index]);
  } else {
    res.status(404).json({ error: 'Todo not found' });
  }
});

// Delete a todo
app.delete('/todos/:id', (req, res) => {
  console.log(req.params.id)
  const id = parseInt(req.params.id);
  const index = todos.findIndex(todo => todo.id === id);
  if (index !== -1) {
    const deletedTodo = todos.splice(index, 1);
    res.json(deletedTodo[0]);
  } else {
    res.status(404).json({ error: 'Todo not found' });
  }
});

// Start the server
const port = 3000;
app.listen(port, () => {
  console.log(`Server is listening on port ${port}`);
});
