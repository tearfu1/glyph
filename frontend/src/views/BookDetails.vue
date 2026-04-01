<script setup lang="ts">
import { useRoute } from 'vue-router'

const route = useRoute()
</script>

<template>
  <div class="max-w-4xl mx-auto px-4 py-12 text-center">
    <h1 class="text-2xl font-bold text-gray-900">Страница книги</h1>
    <p class="mt-2 text-gray-500">ID: {{ route.params.id }}</p>
    <p class="mt-4 text-gray-400">Раздел в разработке</p>
  </div>
</template>
