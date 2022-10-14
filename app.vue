<script setup>
// workaround, can be implemented also with a pinia plugin?
import { useScraperStore } from "~~/stores/scrapers";
const scraperStore = useScraperStore();
await scraperStore.init();
scraperStore.$subscribe((event) => {
  scraperStore.tauriSave();
});
</script>
<template>
  <NuxtLayout>
    <NuxtPage />
  </NuxtLayout>
</template>
