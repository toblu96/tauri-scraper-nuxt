<template>
  <div class="overflow-hidden bg-white shadow sm:rounded-md">
    <ul role="list" class="divide-y divide-gray-200">
      <div class="px-4 py-4 sm:px-6">
        <h2 class="text-lg font-medium leading-6 text-gray-900">
          Version Scrapers
        </h2>
      </div>
      <!-- empty state -->
      <div v-if="scrapers.length == 0" class="px-4 py-12 sm:px-6">
        <div class="text-center">
          <DocumentTextIcon
            class="mx-auto h-12 w-12 stroke-1 text-gray-400"
            aria-hidden="true"
          />
          <h3 class="mt-2 text-sm font-medium text-gray-900">
            No files activated
          </h3>
          <p class="mt-1 text-sm text-gray-500">
            Configure and activate at least one file to get some version changes
            monitored.
          </p>
          <div class="mt-6">
            <NuxtLink
              to="/scrapers"
              class="inline-flex items-center rounded-md border border-transparent bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
            >
              File Configuration
              <ArrowRightIcon class="-mr-1 ml-2 h-5 w-5" aria-hidden="true" />
            </NuxtLink>
          </div>
        </div>
      </div>
      <!-- files list -->
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
                    scraper.update_state === 'Success'
                      ? 'bg-green-100 text-green-800'
                      : 'bg-red-100 text-red-800'
                  "
                  class="truncate rounded-full px-2 text-xs font-semibold leading-5"
                >
                  {{ scraper.update_state || "no data" }}
                </p>
              </div>
            </div>
            <div class="mt-2 sm:flex sm:justify-between">
              <div class="lg:flex">
                <div
                  class="flex w-full items-center text-sm text-gray-500 sm:w-52 sm:max-w-sm md:w-full"
                >
                  <HashtagIcon
                    class="mr-1.5 h-5 w-5 flex-shrink-0 text-gray-400"
                    aria-hidden="true"
                  />
                  <p class="truncate">
                    {{ scraper.last_version || "no data" }}
                  </p>
                </div>
                <p
                  class="mt-2 flex items-center text-sm text-gray-500 lg:mt-0 lg:ml-6"
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
                  <time :datetime="scraper.last_update_utc">{{
                    new Date(scraper.last_update_utc as string).toLocaleString()
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
  ArrowRightIcon,
} from "@heroicons/vue/20/solid";
import { DocumentTextIcon } from "@heroicons/vue/24/outline";

type ScraperProps = {
  id: string;
  name: string;
  enabled: boolean;
  path: string;
  last_update_utc?: string;
  update_state?: string;
  last_version?: string;
};

const props = defineProps<{
  scrapers: ScraperProps[];
}>();
</script>
