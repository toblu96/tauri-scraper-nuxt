<script setup lang="ts">
import { useScraperStore } from "~~/stores/scrapers";
// force loading the store so pinia plugin gets initialized (workaroud due to plugin init too late on page refresh)
const store = useScraperStore();
</script>
<template>
  <NuxtLayout>
    <NuxtPage />
  </NuxtLayout>
</template>
