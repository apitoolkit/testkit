# api-workflows



# How to run 

```rs
cargo run -- --file ./test.yaml
```


## Cypress vs API-workflows
Let's rewrite the following cypress test 
```js

describe('TODO api testing', () => {
   let todoItem;
   it('fetches Todo items - GET', () => {
       cy.request('/todos/').as('todoRequest');
       cy.get('@todoRequest').then(todos => {
          todoItem = todos.body[0]['_id']
           expect(todos.status).to.eq(200);
           assert.isArray(todos.body, 'Todos Response is an array')
       });
   });

   it('deletes Todo items - DELETE', () => {
       cy.request('DELETE', `/todos/${todoItem}`).as('todoRequest');
       cy.get('@todoRequest').then(todos => {
           expect(todos.status).to.eq(200);
           assert.isString(todos.body, 'todo deleted!')
       });
   });

   it('Adds Todo item - POST', () => {
       cy.request('POST', '/todos/', { task: "run tests" }).as('todoRequest');
       // adds new Todo item by defining Todo name
       cy.get('@todoRequest').then(todos => {
           expect(todos.status).to.eq(200);
           cy.wrap(todos.body).should('deep.include', {
               task: 'run tests',
               completed: false,
           });
       });
   });
});

```

```yaml
---
- name: TODO api testing
  stages:  
    - name: fetches TODO items - GET
      request:
        GET: /todos/
      asserts: # Asserts accepts a list of expressions, usually via json_paths to identify the items being refered to.
        true: $.resp.status_code == 200  # Rely on an expressions libray for parsing expressions
        array: $.resp.body.json
      outputs: # values which should be accesible to future steps. 
        todoItem: $.resp.body.json[0]._id

    - name: deletes TODO items - DELETE 
      request:
        DELETE: /todos/{{$.stages[0].outputs.todoItem}} # relative syntax exists: $.stages[-1].outputs.todoItem, -1 means one stage before me
      asserts:
        empty: $.resp.body.json.todos
        string: $.resp.body.json

    - name: Adds Todo item - POST
      request:
        POST: /todos/
        json: 
            task: "run tests"
      asserts:
        true: $.resp.status_code == 200 
        true: $.resp.body.json.task == "run tests"
        false: $.resp.body.json.completed
```
