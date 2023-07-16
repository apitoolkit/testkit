from django.http import JsonResponse
from django.views.decorators.csrf import csrf_exempt
from django.urls import path
import json

todos = [
    {'_id': '1', 'task': 'First task', 'completed': False},
    {'_id': '2', 'task': 'Second task', 'completed': True},
    {'_id': '3', 'task': 'Third task', 'completed': False}
]


@csrf_exempt
def todos_list(request):
    if request.method == 'GET':
        return JsonResponse({'todos': todos}, status=200)

    if request.method == 'POST':
        try:
            data = json.loads(request.body)
            task = data.get('task')
            new_todo = {'_id': str(len(todos) + 1),
                        'task': task, 'completed': False}
            todos.append(new_todo)
            return JsonResponse(new_todo, status=201)
        except json.JSONDecodeError:
            return JsonResponse({'error': 'Invalid JSON'}, status=400)


@csrf_exempt
def todos_detail(request, id):
    if request.method == 'DELETE':
        todos[:] = [todo for todo in todos if todo['_id'] != id]
        return JsonResponse({'message': 'Todo deleted!'}, status=200)
