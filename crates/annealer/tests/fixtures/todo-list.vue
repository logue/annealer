<script setup lang="ts">
import { computed, ref } from 'vue';
import TodoItem from './TodoItem.vue';

const props = defineProps<{ title: string; items: { id: number; text: string; done: boolean }[] }>();
const emit = defineEmits<{ (e: 'toggle', id: number): void }>();
const filter = ref<'all' | 'open'>('all');
const visible = computed(() => props.items.filter((item) => filter.value === 'all' || !item.done));
</script>

<template>
  <section class="todo" :class="{ empty: !visible.length }" aria-labelledby="todo-title">
    <h2 id="todo-title" class="todo__title">{{ title }}</h2>
    <select v-model="filter" aria-label="Filter" class="todo__filter" name="filter">
      <option value="all">All</option>
      <option value="open">Open</option>
    </select>
    <ul class="todo__list" role="list" v-if="visible.length">
      <TodoItem
        @toggle="emit('toggle', item.id)" :key="item.id" v-for="item in visible"
        :itemText="item.text"
        :done="item.done"
        data-testid="todo-item"
      ></TodoItem>
    </ul>
    <p v-else class="todo__empty">Nothing to do.</p>
    <button type="button" @click="filter = 'all'" :disabled="filter === 'all'" class="btn" title="Show all"
      >Show all</button>
    <img alt="" src="/check.svg" width="16" height="16" class="todo__icon" aria-hidden="true">
    <slot name="footer" v-bind="$attrs" :count="visible.length"></slot>
  </section>
</template>

<style scoped lang="scss">
.todo {
  margin: 0 auto;
  color: var(--text);
  display: grid;
  -webkit-user-select: none;
  user-select: none;
  gap: 0.5rem;

  &__title { font-weight: 600; font-size: 1.25rem; margin: 0; }

  &.empty { opacity: 0.6; }
}
</style>
