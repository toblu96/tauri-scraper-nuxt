<script setup lang="ts">
import { ArrowTopRightOnSquareIcon } from "@heroicons/vue/20/solid";
import { Command } from "@tauri-apps/api/shell";

const openLogDirectory = async () => {
  const output = await new Command("open-log-directory").execute();
  console.log(output);
};
</script>
<template>
  <div class="bg-white px-4 py-5 shadow sm:rounded-lg sm:px-6">
    <div class="max-w-xl text-sm text-gray-500">
      <p class="text-sm font-medium text-gray-900">Log Files</p>
      <p>
        All important changes or failures get logged to a logfile on the system.
        Check the logs manually by opening one of the files in the logs
        location.
      </p>
    </div>
    <div class="mt-5">
      <button
        type="button"
        @click="openLogDirectory"
        class="inline-flex items-center rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
      >
        Go to Log Files
        <ArrowTopRightOnSquareIcon
          class="ml-2 -mr-0.5 h-4 w-4"
          aria-hidden="true"
        />
      </button>
    </div>
  </div>
</template>
