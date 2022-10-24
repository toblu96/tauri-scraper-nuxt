<script setup>
import { DocumentIcon } from "@heroicons/vue/24/solid";
import { open } from "@tauri-apps/api/dialog";
import { appDir } from "@tauri-apps/api/path";
import { invoke } from "@tauri-apps/api";

import { useScraperStore } from "~~/stores/scrapers";
const store = useScraperStore();

const test = async () => {
  await invoke("plugin:mqtt-client|connect", {
    clientId: store.mqttBroker.clientId,
    host: store.mqttBroker.host,
    port: store.mqttBroker.port,
  });
};

const path = ref("");
const fileVersion = ref("");
const changeFilePath = async (event) => {
  // Open a selection dialog for directories
  const selected = await open({
    directory: false,
    multiple: false,
    defaultPath: await appDir(),
  });
  if (selected === null) {
    // user cancelled the selection
    console.log("aborted file selection");
  } else {
    // user selected a single directory
    path.value = selected;

    // get file version
    fileVersion.value = await invoke("plugin:file-version|get_file_version", {
      path: path.value,
    });
  }
};
</script>
<template>
  <div class="px-6 pt-8">
    <p>Hello from Home</p>

    <button class="rounded bg-red-300 py-2 px-4" @click="test">init</button>
    <button
      class="rounded bg-red-300 py-2 px-4"
      @click="invoke('plugin:mqtt-client|publish')"
    >
      publish
    </button>

    <div>
      <div class="col-span-6">
        <div>
          <label
            for="scraper-file-path"
            class="block text-sm font-medium text-gray-700"
            >File path</label
          >
          <div class="mt-1 flex rounded-md shadow-sm">
            <div
              class="relative flex flex-grow items-stretch focus-within:z-10"
            >
              <div
                class="pointer-events-none absolute inset-y-0 left-0 flex items-center pl-3"
              >
                <DocumentIcon
                  class="h-5 w-5 text-gray-400"
                  aria-hidden="true"
                />
              </div>
              <input
                type="text"
                v-model="path"
                name="scraper-file-path"
                id="scraper-file-path"
                class="block w-full rounded-none rounded-l-md border-gray-300 pl-10 focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
                placeholder="C:\\Users\\...\\wach_imed.txt"
              />
            </div>
            <button
              type="button"
              @click="changeFilePath"
              class="relative -ml-px inline-flex items-center space-x-2 rounded-r-md border border-gray-300 bg-gray-50 px-4 py-2 text-sm font-medium text-gray-700 hover:bg-gray-100 focus:border-indigo-500 focus:outline-none focus:ring-1 focus:ring-indigo-500"
            >
              <span>Change</span>
            </button>
          </div>
        </div>
      </div>
      <p>{{ fileVersion }}</p>
    </div>
  </div>
</template>
