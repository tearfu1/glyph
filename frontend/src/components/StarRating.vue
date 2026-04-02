<script setup lang="ts">
const props = defineProps<{
  modelValue?: number
  readonly?: boolean
  size?: 'sm' | 'md' | 'lg'
}>()

const emit = defineEmits<{ 'update:modelValue': [value: number] }>()

const sizeClass = {
  sm: 'w-3.5 h-3.5',
  md: 'w-5 h-5',
  lg: 'w-7 h-7',
}

const starSize = sizeClass[props.size ?? 'md']
</script>

<template>
  <div class="flex" :class="readonly ? 'gap-0.5' : 'gap-1'">
    <component
      :is="readonly ? 'span' : 'button'"
      v-for="i in 5"
      :key="i"
      @click="!readonly && emit('update:modelValue', i)"
      :class="[
        !readonly && 'cursor-pointer p-0.5 rounded hover:scale-110 transition-transform',
      ]"
    >
      <svg
        xmlns="http://www.w3.org/2000/svg"
        :class="[
          starSize,
          'transition-colors',
          i <= (modelValue ?? 0) ? 'text-amber-400' : readonly ? 'text-gray-300' : 'text-gray-300 hover:text-amber-200',
        ]"
        viewBox="0 0 24 24"
        fill="currentColor"
      >
        <path fill-rule="evenodd" d="M10.788 3.21c.448-1.077 1.976-1.077 2.424 0l2.082 5.006 5.404.434c1.164.093 1.636 1.545.749 2.305l-4.117 3.527 1.257 5.273c.271 1.136-.964 2.033-1.96 1.425L12 18.354 7.373 21.18c-.996.608-2.231-.29-1.96-1.425l1.257-5.273-4.117-3.527c-.887-.76-.415-2.212.749-2.305l5.404-.434 2.082-5.005Z" clip-rule="evenodd" />
      </svg>
    </component>
  </div>
</template>
