<script setup lang="ts">
import { ArrowTopRightOnSquareIcon } from "@heroicons/vue/20/solid";
import { open } from "@tauri-apps/api/shell";
const tauriStore = useTauriStore();
</script>
<template>
  <div class="bg-white px-4 py-5 shadow sm:rounded-lg sm:px-6">
    <div class="max-w-xl text-sm text-gray-500">
      <p class="text-sm font-medium text-gray-900">
        Backend OpenAPI Documentation
      </p>
      <p>
        There is a globally available backend server, which takes care of the
        file changes and sends them to other systems. Check the OpenAPI specs
        here.
      </p>
    </div>
    <div class="mt-5">
      <button
        type="button"
        @click="open('http://localhost:8000')"
        class="inline-flex items-center rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
      >
        Open OpenAPI Specs
        <ArrowTopRightOnSquareIcon
          class="ml-2 -mr-0.5 h-4 w-4"
          aria-hidden="true"
        />
      </button>
    </div>
  </div>
</template>
