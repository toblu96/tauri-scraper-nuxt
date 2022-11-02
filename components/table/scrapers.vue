<script setup lang="ts">
import { Ref, ref, computed } from "vue";
import { Switch } from "@headlessui/vue";
import { useScraperStore } from "~~/stores/scrapers";
import { DocumentPlusIcon } from "@heroicons/vue/24/outline";
import { save, message, open } from "@tauri-apps/api/dialog";
import { writeTextFile, readTextFile } from "@tauri-apps/api/fs";

const scraperStore = useScraperStore();

type ScraperProps = {
  id: string;
  name: string;
  enabled: boolean;
  lastUpdate?: string;
  path: string;
};

type ImportedScraperProps = {
  name: string;
  path: string;
  mqttTopic: string;
};

const props = defineProps<{
  scrapers: ScraperProps[];
}>();

const selectedScrapers: Ref<string[]> = ref([]);
const checked = ref(false);
const indeterminate = computed(
  () =>
    selectedScrapers.value.length > 0 &&
    selectedScrapers.value.length < props.scrapers.length
);

const addScraper = () => {
  scraperStore.addFileScraper({
    enabled: false,
    name: "quick scraper",
    path: "none",
    mqttTopic: "eh/test/1234",
  });
};
const deleteScrapers = () => {
  for (const scraperIdx of selectedScrapers.value) {
    scraperStore.removeFileScraper(scraperIdx);
  }
  selectedScrapers.value = [];
};
const exportScrapers = async () => {
  try {
    // open save dialog
    const filePath = await save({
      defaultPath: "settings",
      filters: [
        {
          name: "json",
          extensions: ["json"],
        },
      ],
    });
    if (filePath === null) {
      // user cancelled the selection
      console.log("aborted file selection");
      return;
    }
    // filter content from selected scrapers and drop unnecessary keys
    const data = scraperStore.scraperList
      .filter((scraper) => selectedScrapers.value.includes(scraper.id))
      .map(
        ({
          enabled,
          stop,
          id,
          lastUpdateUTC,
          lastVersion,
          updateState,
          ...keepAttrs
        }) => keepAttrs
      );
    // write selected scrapers to file
    await writeTextFile(filePath, JSON.stringify(data));
  } catch (error) {
    message(`Could not save settings: \n${error}`, {
      title: "Tauri | Save file scraper settings",
      type: "warning",
    });
  }
};
const importScrapers = async () => {
  try {
    // get file path from settings file
    const filePath = (await open({
      directory: false,
      multiple: false,
      filters: [
        {
          name: "json",
          extensions: ["json"],
        },
      ],
    })) as string;
    if (filePath === null) {
      // user cancelled the selection
      console.log("aborted file selection");
      return;
    }
    // open settings file and extract config
    const files: ImportedScraperProps[] = JSON.parse(
      await readTextFile(filePath)
    );
    // add files to store
    for (const file of files) {
      if (!file.name || !file.path || !file.mqttTopic) {
        await message(
          `Scraper is missing some params: \n
            name:\t ${file.name || "-"}
            path:\t ${file.path || "-"}
            mqttTopic:\t ${file.mqttTopic || "-"}\n` +
            `\nSkip that one and try next.`,
          {
            title: "Tauri | File scraper import",
            type: "warning",
          }
        );
        continue;
      }
      scraperStore.addFileScraper({
        enabled: false,
        name: file.name || "Default name",
        path: file.path || "",
        mqttTopic: file.mqttTopic || "",
      });
    }
  } catch (error) {
    message(`Could not import scrapers: \n${error}`, {
      title: "Tauri | File scraper import",
      type: "warning",
    });
  }
};
</script>

<template>
  <div class="sm:flex sm:items-center">
    <div class="sm:flex-auto">
      <!-- <h1 class="text-xl font-semibold text-gray-900">File Scraper</h1> -->
      <p class="mt-2 text-sm text-gray-700">
        A list of all the file scraper on the current machine including some
        quick actions.
      </p>
    </div>
    <div class="mt-4 space-x-4 sm:mt-0 sm:ml-16 sm:flex-none">
      <button
        @click="importScrapers()"
        type="button"
        class="inline-flex items-center rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
      >
        Import
      </button>
      <button
        @click="addScraper()"
        type="button"
        class="inline-flex items-center justify-center rounded-md border border-transparent bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 sm:w-auto"
      >
        Add scraper
      </button>
    </div>
  </div>
  <div class="mt-8 flex flex-col">
    <div class="-my-2 -mx-4 overflow-x-auto sm:-mx-6 lg:-mx-8">
      <div class="inline-block min-w-full py-2 align-middle md:px-6 lg:px-8">
        <div
          class="relative overflow-hidden shadow ring-1 ring-black ring-opacity-5 md:rounded-lg"
        >
          <div
            v-if="selectedScrapers.length > 0"
            class="absolute top-0 left-12 flex h-12 items-center space-x-3 bg-gray-50 sm:left-16"
          >
            <button
              @click="exportScrapers()"
              type="button"
              class="inline-flex items-center rounded border border-gray-300 bg-white px-2.5 py-1.5 text-xs font-medium text-gray-700 shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-30"
            >
              Export
            </button>
            <button
              @click="deleteScrapers()"
              type="button"
              class="inline-flex items-center rounded border border-gray-300 bg-white px-2.5 py-1.5 text-xs font-medium text-gray-700 shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 disabled:cursor-not-allowed disabled:opacity-30"
            >
              Delete all
            </button>
          </div>
          <table class="min-w-full table-fixed divide-y divide-gray-300">
            <thead class="bg-gray-50">
              <tr>
                <th scope="col" class="relative w-12 px-6 sm:w-16 sm:px-8">
                  <input
                    type="checkbox"
                    class="absolute left-4 top-1/2 -mt-2 h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500 sm:left-6"
                    :checked="
                      (indeterminate ||
                        selectedScrapers.length === scrapers.length) &&
                      selectedScrapers.length > 0
                    "
                    :indeterminate="indeterminate"
                    @change="
                      selectedScrapers = $event.target.checked
                        ? scrapers.map((p) => p.id)
                        : []
                    "
                  />
                </th>
                <th
                  scope="col"
                  class="min-w-[8rem] py-3.5 pr-3 text-left text-sm font-semibold text-gray-900"
                >
                  Name
                </th>
                <th
                  scope="col"
                  class="px-3 py-3.5 text-left text-sm font-semibold text-gray-900"
                >
                  File path
                </th>
                <th
                  scope="col"
                  class="px-3 py-3.5 text-left text-sm font-semibold text-gray-900 lg:w-24"
                >
                  Enabled
                </th>
                <th
                  scope="col"
                  class="relative py-3.5 pl-3 pr-4 sm:pr-6 lg:w-24"
                >
                  <span class="sr-only">Edit</span>
                </th>
              </tr>
            </thead>
            <tbody class="divide-y divide-gray-200 bg-white">
              <tr
                v-for="scraper in scrapers"
                :key="scraper.id"
                :class="[selectedScrapers.includes(scraper.id) && 'bg-gray-50']"
              >
                <td class="relative w-12 px-6 sm:w-16 sm:px-8">
                  <div
                    v-if="selectedScrapers.includes(scraper.id)"
                    class="absolute inset-y-0 left-0 w-0.5 bg-indigo-600"
                  ></div>
                  <input
                    type="checkbox"
                    class="absolute left-4 top-1/2 -mt-2 h-4 w-4 rounded border-gray-300 text-indigo-600 focus:ring-indigo-500 sm:left-6"
                    :value="scraper.id"
                    v-model="selectedScrapers"
                  />
                </td>
                <td
                  :class="[
                    'whitespace-nowrap py-4 pr-3 text-sm font-medium',
                    selectedScrapers.includes(scraper.id)
                      ? 'text-indigo-600'
                      : 'text-gray-900',
                  ]"
                >
                  {{ scraper.name }}
                </td>
                <td class="whitespace-nowrap px-3 py-4 text-sm text-gray-500">
                  {{ scraper.path }}
                </td>
                <td class="whitespace-nowrap py-4 text-sm text-gray-500">
                  <Switch
                    v-model="scraper.enabled"
                    @click="scraperStore.toggleEnableState(scraper.id)"
                    :class="[
                      scraper.enabled ? 'bg-indigo-500' : 'bg-gray-200',
                      'relative ml-4 inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2',
                    ]"
                  >
                    <span
                      aria-hidden="true"
                      :class="[
                        scraper.enabled ? 'translate-x-5' : 'translate-x-0',
                        'inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                      ]"
                    />
                  </Switch>
                </td>
                <td
                  class="whitespace-nowrap py-4 pl-3 pr-4 text-right text-sm font-medium sm:pr-6"
                >
                  <NuxtLink
                    :to="`/scrapers/${scraper.id}/edit`"
                    class="text-indigo-600 hover:text-indigo-900"
                  >
                    Edit
                    <span class="sr-only"> {{ scraper.name }}</span></NuxtLink
                  >
                </td>
              </tr>
            </tbody>
          </table>
          <div class="bg-white px-4 py-12" v-if="scrapers.length <= 0">
            <div class="mx-auto max-w-lg">
              <button
                @click="addScraper()"
                type="button"
                class="relative block w-full rounded-lg border-2 border-dashed border-gray-300 p-12 text-center hover:border-gray-400 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2"
              >
                <DocumentPlusIcon
                  class="mx-auto h-12 w-12 stroke-1 text-gray-400"
                  aria-hidden="true"
                />

                <span class="mt-2 block text-sm font-medium text-gray-900"
                  >Create a new file config</span
                >
              </button>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>
