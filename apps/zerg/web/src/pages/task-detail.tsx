import { createQuery, createMutation, useQueryClient } from '@tanstack/solid-query';
import { useParams, useNavigate } from '@tanstack/solid-router';
import { Show, createSignal } from 'solid-js';
import { tasksApi, type UpdateTaskInput } from '../lib/api-client';

export function TaskDetailPage() {
  const params = useParams({ from: '/tasks/$id' });
  const navigate = useNavigate();
  const queryClient = useQueryClient();

  const taskQuery = createQuery(() => ({
    queryKey: ['tasks', params.id],
    queryFn: () => tasksApi.getById(params.id),
  }));

  const updateMutation = createMutation(() => ({
    mutationFn: ({ id, input }: { id: string; input: UpdateTaskInput }) =>
      tasksApi.update(id, input),
    onSuccess: () => {
      queryClient.invalidateQueries({ queryKey: ['tasks'] });
      queryClient.invalidateQueries({ queryKey: ['tasks', params.id] });
    },
  }));

  const [editing, setEditing] = createSignal(false);
  const [formData, setFormData] = createSignal<UpdateTaskInput>({});

  const handleEdit = () => {
    const task = taskQuery.data;
    if (task) {
      setFormData({
        title: task.title,
        description: task.description,
        status: task.status,
        priority: task.priority,
      });
      setEditing(true);
    }
  };

  const handleSave = () => {
    updateMutation.mutate(
      { id: params.id, input: formData() },
      {
        onSuccess: () => {
          setEditing(false);
        },
      }
    );
  };

  return (
    <div class="container mx-auto p-4 max-w-4xl">
      <Show
        when={!taskQuery.isLoading && taskQuery.data}
        fallback={<div class="text-center py-8">Loading...</div>}
      >
        {(task) => (
          <div>
            <div class="flex justify-between items-center mb-6">
              <button
                onClick={() => navigate({ to: '/tasks' })}
                class="text-blue-500 hover:text-blue-700"
              >
                ← Back to Tasks
              </button>
              <Show when={!editing()}>
                <button
                  onClick={handleEdit}
                  class="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600"
                >
                  Edit
                </button>
              </Show>
            </div>

            <Show
              when={editing()}
              fallback={
                <div class="bg-white rounded-lg shadow p-6">
                  <h1 class="text-3xl font-bold mb-4">{task().title}</h1>
                  <p class="text-gray-700 mb-4">{task().description}</p>
                  <div class="flex gap-4 mb-4 flex-wrap">
                    <div>
                      <span class="font-semibold">Status: </span>
                      <span class="capitalize">{task().status.replace('_', ' ')}</span>
                    </div>
                    <div>
                      <span class="font-semibold">Priority: </span>
                      <span class="capitalize">{task().priority}</span>
                    </div>
                    <div>
                      <span class="font-semibold">Completed: </span>
                      <span>{task().completed ? 'Yes' : 'No'}</span>
                    </div>
                  </div>
                  {task().due_date && (
                    <div class="mb-4">
                      <span class="font-semibold">Due Date: </span>
                      <span>{new Date(task().due_date!).toLocaleDateString()}</span>
                    </div>
                  )}
                  <div class="text-sm text-gray-500 space-y-1">
                    <div>Created: {new Date(task().created_at).toLocaleString()}</div>
                    <div>Updated: {new Date(task().updated_at).toLocaleString()}</div>
                  </div>
                </div>
              }
            >
              <div class="bg-white rounded-lg shadow p-6 space-y-4">
                <div>
                  <label class="block font-semibold mb-1">Title</label>
                  <input
                    type="text"
                    value={formData().title || ''}
                    onInput={(e) => setFormData({ ...formData(), title: e.currentTarget.value })}
                    class="w-full border rounded px-3 py-2"
                  />
                </div>
                <div>
                  <label class="block font-semibold mb-1">Description</label>
                  <textarea
                    value={formData().description || ''}
                    onInput={(e) => setFormData({ ...formData(), description: e.currentTarget.value })}
                    class="w-full border rounded px-3 py-2"
                    rows={4}
                  />
                </div>
                <div class="flex gap-4">
                  <div class="flex-1">
                    <label class="block font-semibold mb-1">Status</label>
                    <select
                      value={formData().status || ''}
                      onChange={(e) => setFormData({ ...formData(), status: e.currentTarget.value as any })}
                      class="w-full border rounded px-3 py-2"
                    >
                      <option value="todo">To Do</option>
                      <option value="in_progress">In Progress</option>
                      <option value="done">Done</option>
                    </select>
                  </div>
                  <div class="flex-1">
                    <label class="block font-semibold mb-1">Priority</label>
                    <select
                      value={formData().priority || ''}
                      onChange={(e) => setFormData({ ...formData(), priority: e.currentTarget.value as any })}
                      class="w-full border rounded px-3 py-2"
                    >
                      <option value="low">Low</option>
                      <option value="medium">Medium</option>
                      <option value="high">High</option>
                      <option value="urgent">Urgent</option>
                    </select>
                  </div>
                </div>
                <div class="flex items-center gap-2">
                  <input
                    type="checkbox"
                    checked={formData().completed || false}
                    onChange={(e) => setFormData({ ...formData(), completed: e.currentTarget.checked })}
                    class="rounded"
                  />
                  <label>Mark as completed</label>
                </div>
                <div class="flex gap-2">
                  <button
                    onClick={handleSave}
                    disabled={updateMutation.isPending}
                    class="bg-blue-500 text-white px-4 py-2 rounded hover:bg-blue-600 disabled:opacity-50"
                  >
                    {updateMutation.isPending ? 'Saving...' : 'Save'}
                  </button>
                  <button
                    onClick={() => setEditing(false)}
                    class="bg-gray-300 px-4 py-2 rounded hover:bg-gray-400"
                  >
                    Cancel
                  </button>
                </div>
              </div>
            </Show>
          </div>
        )}
      </Show>
    </div>
  );
}
