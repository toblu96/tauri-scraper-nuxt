<script setup>
import { useScraperStore } from "~~/stores/scrapers";
const store = useScraperStore();
</script>
<template>
  <div class="space-y-6 px-6 pt-8">
    <PanelMqttBroker />
    <ListScraperInfo :scrapers="store.enabledFileScrapers" />
  </div>
</template>
