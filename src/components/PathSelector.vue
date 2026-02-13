<template>
  <div class="flex flex-col gap-1 w-full">
    <label
        class="text-xs font-semibold text-gray-500 dark:text-gray-400 tracking-wider truncate block cursor-help"
        :title="label"
    >
      {{ shortLabel }}
    </label>
    <div class="flex gap-2">
      <input
          type="text"
          :value="modelValue"
          :placeholder="placeholder"
          :disabled="disabled"
          @click="handleBrowse"
          @mouseenter="showTooltip = true"
          @mouseleave="showTooltip = false"
          @focus="showTooltip = true"
          @blur="showTooltip = false"
          readonly
          class="flex-1 bg-white dark:bg-gray-800 border border-gray-300 dark:border-gray-700 rounded-md px-3 py-2 text-sm text-gray-900 dark:text-gray-100 focus:outline-none focus:ring-2 focus:ring-blue-500 transition-all cursor-pointer relative"
          :title="modelValue || ''"
      />
      <button
          @click="handleBrowse"
          :disabled="disabled"
          class="bg-gray-100 dark:bg-gray-700 hover:bg-gray-200 dark:hover:bg-gray-600 border border-gray-300 dark:border-gray-600 rounded-md px-3 py-2 text-gray-700 dark:text-gray-200 transition-colors flex items-center justify-center disabled:opacity-50 disabled:cursor-not-allowed"
          title="Browse"
      >
        <FolderOpen :size="18"/>
      </button>
      <button
          @click="handleSearch"
          :disabled="disabled"
          class="bg-blue-100 dark:bg-blue-700 hover:bg-blue-200 dark:hover:bg-blue-600 border border-blue-300 dark:border-blue-600 rounded-md px-3 py-2 text-blue-700 dark:text-blue-200 transition-colors flex items-center justify-center disabled:opacity-50 disabled:cursor-not-allowed"
          title="Search"
      >
        <Search :size="18"/>
      </button>
      <button
          @click="handleReset"
          :disabled="disabled"
          class="bg-gray-200 dark:bg-gray-600 hover:bg-gray-300 dark:hover:bg-gray-500 border border-gray-300 dark:border-gray-500 rounded-md px-3 py-2 text-gray-700 dark:text-gray-200 transition-colors flex items-center justify-center disabled:opacity-50 disabled:cursor-not-allowed"
          title="Reset"
      >
        <RotateCcw :size="18"/>
      </button>
    </div>
  </div>
</template>

<script setup>
import {FolderOpen, Search, RotateCcw} from 'lucide-vue-next';
import {computed, ref} from "vue";
import {open} from '@tauri-apps/plugin-dialog';
import {ElMessage} from "element-plus";

const props = defineProps({
  label: String,
  modelValue: String,
  placeholder: String,
  disabled: Boolean,
  // 可以通过 prop 传入长度，不传默认 10
  maxLength: {
    type: Number,
    default: 10
  }
});

// 控制工具提示显示
const showTooltip = ref(false);

// 计算显示的文字
const shortLabel = computed(() => {
  if (!props.label) return '';
  // 如果长度超过限制，截取前 N 个字符并加 ...
  if (props.label.length > props.maxLength) {
    return props.label.substring(0, props.maxLength) + '...';
  }
  return props.label;
});

const emit = defineEmits(['search', 'reset']);

const handleBrowse = async () => {
  if (props.disabled) return;

  try {
    // 打开文件夹选择对话框
    const selected = await open({
      directory: true,     // 只允许选择文件夹
      multiple: false,     // 不允许多选
      defaultPath: props.modelValue || undefined, // 打开时默认定位到当前路径
      title: 'Select a folder'
    });

    // open 返回 string | string[] | null
    // 因为 multiple: false，所以返回 string 或 null
    if (selected && typeof selected === 'string') {
      emit('update:modelValue', selected);
    }
  } catch (error) {
    ElMessage.error('Failed to open folder dialog.')
    console.error('Failed to open folder dialog:', error);
  }
};

const handleSearch = () => {
  if (props.disabled) return;
  emit('search');
};

const handleReset = () => {
  if (props.disabled) return;
  emit('reset');
};
</script>