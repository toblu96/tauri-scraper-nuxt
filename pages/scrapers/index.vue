<script setup>
import { useScraperStore } from "~~/stores/scrapers";
const store = useScraperStore();
</script>

<template>
  <div class="px-6 pt-8">
    <TableScrapers :scrapers="store.fileScrapers" />
  </div>
</template>
