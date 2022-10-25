<script setup>
import { useScraperStore } from "~~/stores/scrapers";
const route = useRoute();

const scraperStore = useScraperStore();
const scraper = scraperStore.fileScrapers.find(
  (scraper) => scraper.id === route.params.id
);
</script>
<template>
  <form class="space-y-6" action="#" method="POST">
    <div class="bg-white px-4 py-5 shadow sm:rounded-lg sm:p-6">
      <div class="md:grid md:grid-cols-3 md:gap-6">
        <div class="md:col-span-1">
          <h3 class="text-lg font-medium leading-6 text-gray-900">MQTT</h3>
          <p class="mt-1 text-sm text-gray-500">
            Specify the MQTT topic to publish the version to.
          </p>
        </div>
        <div class="mt-5 md:col-span-2 md:mt-0">
          <div class="grid grid-cols-6 gap-6">
            <div class="col-span-6">
              <label
                for="scraper-topic"
                class="block text-sm font-medium text-gray-700"
                >Topic</label
              >
              <input
                type="text"
                v-model="scraper.mqttTopic"
                name="scraper-topic"
                id="scraper-topic"
                placeholder="eh/test/topic"
                class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
              />
            </div>
          </div>
        </div>
      </div>
    </div>
  </form>
</template>
