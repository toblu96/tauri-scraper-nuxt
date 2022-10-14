<script setup>
import { useScrapersStore } from "~~/store/scrapers";
const store = useScrapersStore();
</script>

<template>
  <div class="own px-6 pt-8">
    <TableScrapers :scrapers="store.fileScrapers" />
  </div>
</template>
