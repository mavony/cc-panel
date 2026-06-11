<script setup lang="ts">
import type { TodoItem } from "../../types";

defineProps<{ todos: TodoItem[] }>();
</script>

<template>
  <ul class="todos">
    <li
      v-for="(todo, i) in todos"
      :key="i"
      class="todo"
      :class="`is-${todo.status}`"
    >
      <span class="todo-mark">
        <template v-if="todo.status === 'completed'">✓</template>
        <template v-else-if="todo.status === 'in_progress'">●</template>
      </span>
      <span class="todo-text">{{ todo.content }}</span>
    </li>
  </ul>
</template>

<style scoped>
.todos {
  margin: 0;
  padding: 0;
  list-style: none;
  display: flex;
  flex-direction: column;
  gap: 4px;
}

.todo {
  display: flex;
  align-items: baseline;
  gap: 8px;
  font-size: 12px;
}

.todo-mark {
  flex-shrink: 0;
  width: 14px;
  height: 14px;
  border: 1px solid #3a414d;
  border-radius: 50%;
  display: inline-flex;
  align-items: center;
  justify-content: center;
  font-size: 9px;
  transform: translateY(2px);
}

.is-completed .todo-mark {
  background: #2f6b4f;
  border-color: #2f6b4f;
  color: #d9f2e5;
}

.is-in_progress .todo-mark {
  border-color: #6ea8fe;
  color: #6ea8fe;
}

.is-completed .todo-text {
  color: #5c6470;
  text-decoration: line-through;
}

.is-in_progress .todo-text {
  color: #e6e9ee;
  font-weight: 500;
}

.todo-text {
  color: #b8bfc9;
}
</style>
