<script setup>
import { emit, listen, once } from "@tauri-apps/api/event";
// workaround, can be implemented also with a pinia plugin?
import { useScraperStore } from "~~/stores/scrapers";
const scraperStore = useScraperStore();
await scraperStore.init();
scraperStore.$subscribe((event) => {
  scraperStore.tauriSave(event);
});
</script>
<template>
  <NuxtLayout>
    <NuxtPage />
  </NuxtLayout>
</template>
