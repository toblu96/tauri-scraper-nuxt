<template>
  <div class="overflow-hidden bg-white shadow sm:rounded-md">
    <ul role="list" class="divide-y divide-gray-200">
      <li class="px-4 py-4 sm:px-6">
        <h2 class="text-lg font-medium leading-6 text-gray-900">
          Version Scrapers
        </h2>
      </li>
      <li v-for="scraper in scrapers" :key="scraper.id">
        <NuxtLink
          :to="`/scrapers/${scraper.id}/edit`"
          class="block hover:bg-gray-50"
        >
          <div class="px-4 py-4 sm:px-6">
            <div class="flex items-center justify-between">
              <p class="flex-none text-sm font-medium text-indigo-600">
                {{ scraper.name }}
              </p>
              <div class="ml-2 flex flex-shrink-0">
                <p
                  :class="
                    scraper.updateState === 'Successful'
                      ? 'bg-green-100 text-green-800'
                      : 'bg-red-100 text-red-800'
                  "
                  class="truncate rounded-full px-2 text-xs font-semibold leading-5"
                >
                  {{ scraper.updateState || "no data" }}
                </p>
              </div>
            </div>
            <div class="mt-2 sm:flex sm:justify-between">
              <div class="sm:flex">
                <p class="flex items-center text-sm text-gray-500">
                  <HashtagIcon
                    class="mr-1.5 h-5 w-5 flex-shrink-0 text-gray-400"
                    aria-hidden="true"
                  />
                  {{ scraper.lastVersion || "no data" }}
                </p>
                <p
                  class="mt-2 flex items-center text-sm text-gray-500 sm:mt-0 sm:ml-6"
                >
                  <DocumentMagnifyingGlassIcon
                    class="mr-1.5 h-5 w-5 flex-shrink-0 text-gray-400"
                    aria-hidden="true"
                  />
                  {{ scraper.path }}
                </p>
              </div>
              <div class="mt-2 flex items-center text-sm text-gray-500 sm:mt-0">
                <ClockIcon
                  class="mr-1.5 h-5 w-5 flex-shrink-0 text-gray-400"
                  aria-hidden="true"
                />
                <p>
                  Last updated on
                  {{ " " }}
                  <time :datetime="scraper.lastUpdateUTC">{{
                    new Date(scraper.lastUpdateUTC).toLocaleString()
                  }}</time>
                </p>
              </div>
            </div>
          </div>
        </NuxtLink>
      </li>
    </ul>
  </div>
</template>

<script setup lang="ts">
import {
  ClockIcon,
  HashtagIcon,
  DocumentMagnifyingGlassIcon,
} from "@heroicons/vue/20/solid";

type ScraperProps = {
  id: string;
  name: string;
  enabled: boolean;
  path: string;
  lastUpdateUTC?: string;
  updateState?: string;
  lastVersion?: string;
};

const props = defineProps<{
  scrapers: ScraperProps[];
}>();
</script>
