const express = require('express');
const app = express();
app.use(express.json());

let todos = [
    { _id: '1', task: 'First task', completed: false },
    { _id: '2', task: 'Second task', completed: true },
    { _id: '3', task: 'Third task', completed: false }
];

app.get('/todos', (req, res) => {
    res.status(200).json({ todos });
});

app.delete('/todos/:id', (req, res) => {
    const id = req.params.id;
    todos = todos.filter(todo => todo._id !== id);
    res.status(200).json({ message: 'Todo deleted!' });
});

app.post('/todos', (req, res) => {
    const task = req.body.task;
    const newTodo = { _id: todos.length + 1, task, completed: false };
    todos.push(newTodo);
    res.status(201).json(newTodo);
});

app.listen(3000, () => {
    console.log('Server is running on port 3000');
});
