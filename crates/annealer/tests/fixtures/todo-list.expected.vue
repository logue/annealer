<script setup lang="ts">
import { computed, ref } from 'vue';
import TodoItem from './TodoItem.vue';

const props = defineProps<{
  title: string;
  items: { id: number; text: string; done: boolean }[];
}>();
const emit = defineEmits<{ (e: 'toggle', id: number): void }>();
const filter = ref<'all' | 'open'>('all');
const visible = computed(() =>
  props.items.filter(item => filter.value === 'all' || !item.done)
);
</script>

<template>
  <section
    class="todo"
    :class="{ empty: !visible.length }"
    aria-labelledby="todo-title"
  >
    <h2 id="todo-title" class="todo__title">{{ title }}</h2>
    <select
      v-model="filter"
      class="todo__filter"
      name="filter"
      aria-label="Filter"
    >
      <option value="all">All</option>
      <option value="open">Open</option>
    </select>
    <ul v-if="visible.length" class="todo__list" role="list">
      <TodoItem
        v-for="item in visible"
        :key="item.id"
        :item-text="item.text"
        :done="item.done"
        data-testid="todo-item"
        @toggle="emit('toggle', item.id)"
      />
    </ul>
    <p v-else class="todo__empty">Nothing to do.</p>
    <button
      class="btn"
      title="Show all"
      type="button"
      :disabled="filter === 'all'"
      @click="filter = 'all'"
    >
      Show all
    </button>
    <img
      class="todo__icon"
      alt=""
      src="/check.svg"
      width="16"
      height="16"
      aria-hidden="true"
    />
    <slot name="footer" v-bind="$attrs" :count="visible.length" />
  </section>
</template>

<style scoped lang="scss">
.todo {
  -webkit-user-select: none;
  display: grid;
  margin: 0 auto;
  gap: 0.5rem;
  color: var(--text);
  user-select: none;

  &__title {
    margin: 0;
    font-weight: 600;
    font-size: 1.25rem;
  }

  &.empty {
    opacity: 0.6;
  }
}
</style>
