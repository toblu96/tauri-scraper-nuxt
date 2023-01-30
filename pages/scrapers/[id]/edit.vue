<script setup>
import { ArrowLeftIcon } from "@heroicons/vue/24/solid";
import { TrashIcon } from "@heroicons/vue/24/outline";

const route = useRoute();

const deleteFile = async () => {
  // delete one file by its id and navigate back to file overview
  let res = await useFetch(
    `http://localhost:8000/api/files/${route.params.id}`,
    {
      method: "DELETE",
    }
  );
  if (res.error.value) {
    console.error(res.error.value);
  } else {
    navigateTo("/scrapers");
  }
};
</script>
<template>
  <div class="space-y-6 px-6 py-8">
    <div class="flex justify-between">
      <NuxtLink
        to="/scrapers"
        class="inline-flex items-center rounded-md border border-gray-300 bg-white px-3 py-2 text-sm font-medium leading-4 text-gray-700 shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
      >
        <ArrowLeftIcon class="-ml-1 mr-2 h-5 w-5" aria-hidden="true" />
        Back
      </NuxtLink>
      <button
        @click="deleteFile"
        class="inline-flex items-center rounded-md border border-gray-300 bg-white px-3 py-2 text-sm font-medium leading-4 text-gray-700 shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
      >
        <TrashIcon class="h-5 w-5" aria-hidden="true" />
      </button>
    </div>
    <FormScraperSettingsFile />
    <FormScraperSettingsMqtt />
  </div>
</template>
