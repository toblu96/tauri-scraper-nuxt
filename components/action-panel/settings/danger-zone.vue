<script setup>
const tauriStore = useTauriStore();
</script>
<template>
  <div class="border border-red-500 bg-white sm:rounded-lg">
    <div class="px-4 py-5 sm:p-6">
      <h3 class="text-lg font-medium leading-6 text-gray-900">
        Delete persistent storage
      </h3>
      <div class="mt-2 max-w-xl text-sm text-gray-500">
        <p>
          Once you delete persistent storage, you will lose all your settings on
          this machine.
        </p>
      </div>
      <div class="mt-5">
        <button
          @click="tauriStore.clear()"
          type="button"
          class="inline-flex items-center justify-center rounded-md border border-transparent bg-red-100 px-4 py-2 font-medium text-red-700 hover:bg-red-200 focus:outline-none focus:ring-2 focus:ring-red-500 focus:ring-offset-2 sm:text-sm"
        >
          Delete storage
        </button>
      </div>
    </div>
  </div>
</template>
