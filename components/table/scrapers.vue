<script setup lang="ts">
import { Ref, ref, computed } from "vue";
import { Switch } from "@headlessui/vue";
import { DocumentPlusIcon } from "@heroicons/vue/24/outline";
import { save, message, open } from "@tauri-apps/api/dialog";
import { writeTextFile, readTextFile } from "@tauri-apps/api/fs";
import { z } from "zod";

interface IFile {
  id: string;
  name: string;
  enabled: boolean;
  last_update_utc?: string; // timestamp UTC
  update_state?: string; // status of update - e.g. could not read | successful
  last_version?: string; // latest file version
  path: string;
  mqtt_topic: string;
}

const zodFileSchema = z.object({
  name: z.string(),
  path: z.string(),
  mqtt_topic: z.string(),
});

// subscribe to active files from sse backend
const files = ref<IFile[]>([]);
let eventSource = new EventSource("http://localhost:8000/api/files/sse");
eventSource.onmessage = function (event) {
  if (blockEventSourceUpdates.value) return;
  console.log("got update");
  try {
    let unsortedFiles: IFile[] = JSON.parse(event.data);
    // sort by path and id
    files.value = unsortedFiles.sort((a, b) => {
      if (a.path == b.path) return a.id.localeCompare(b.id);
      return a.path.localeCompare(b.path);
    });
  } catch (error) {
    console.error(`Could not update files: ${error}`);
  }

  // release locks
  isEnableLocked.value.length = 0;
  isAddFileLocked.value = false;
};
// close eventsource on page leave
onUnmounted(() => {
  eventSource.close();
});

type ImportedScraperProps = {
  name: string;
  path: string;
  mqtt_topic: string;
};

const isAddFileLocked = ref(false);
const blockEventSourceUpdates = ref(false);
const changedFilesPending = ref<string[]>([]);
const isEnableLocked = ref<string[]>([]);

const selectedScrapers: Ref<string[]> = ref([]);
const checked = ref(false);
const indeterminate = computed(
  () =>
    selectedScrapers.value.length > 0 &&
    selectedScrapers.value.length < files.value.length
);

const addScraper = async () => {
  isAddFileLocked.value = true;

  let res = await useFetch(`http://localhost:8000/api/files`, {
    method: "POST",
    body: {
      enabled: false,
      mqtt_topic: "eh/test/topic",
      name: "ExampleFile.dll",
      path: "C:/win/doof",
    },
  });
  if (res.error.value) {
    console.error(res.error.value);
  }
};
const deleteScrapers = async () => {
  for (const fileId of selectedScrapers.value) {
    changedFilesPending.value.push(fileId);

    let res = await useFetch(`http://localhost:8000/api/files/${fileId}`, {
      method: "DELETE",
    });
    if (res.error.value) {
      console.error(res.error.value);
    }
  }
  selectedScrapers.value = [];
};
const handleToggle = async (id: string, state: boolean) => {
  blockEventSourceUpdates.value = true;
  isEnableLocked.value.push(id);
  let res = await useFetch(`http://localhost:8000/api/files/${id}`, {
    method: "PATCH",
    body: {
      enabled: state,
    },
  });
  if (res.error.value) {
    console.error(res.error.value);
  }
  blockEventSourceUpdates.value = false;
};
const exportScrapers = async () => {
  try {
    // open save dialog
    const filePath = await save({
      defaultPath: "files_configuration",
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
    const data = files.value
      .filter((scraper) => selectedScrapers.value.includes(scraper.id))
      .map(
        ({
          enabled,
          id,
          last_update_utc,
          last_version,
          update_state,
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
  let importErrors: string = "";
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
      isAddFileLocked.value = true;

      // validate incoming file structure
      let parsedFile = zodFileSchema.safeParse(file);

      if (!parsedFile.success) {
        console.log(parsedFile.error.message);
        importErrors += `File '${file.name}' \n`;
        for (let msg of parsedFile.error.issues) {
          importErrors += `- '${msg.path[0]}' -> ${msg.message} \n`;
        }
        importErrors += `\n`;
        continue;
      }

      let res = await useFetch(`http://localhost:8000/api/files`, {
        method: "POST",
        body: {
          enabled: false,
          mqtt_topic: file.mqtt_topic,
          name: file.name,
          path: file.path,
        },
      });
      if (res.error.value) {
        console.error(res.error.value);
      }
    }

    // print summarized error message
    if (importErrors != "") {
      await message(`Could not import all files: \n\n ${importErrors}`, {
        title: "Tauri | Files import",
        type: "warning",
      });
    }
  } catch (error) {
    message(`Could not import scrapers: \n${error}`, {
      title: "Tauri | Files import",
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
        :disabled="isAddFileLocked"
        type="button"
        class="inline-flex items-center rounded-md border border-gray-300 bg-white px-4 py-2 text-sm font-medium text-gray-700 shadow-sm hover:bg-gray-50 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 disabled:animate-pulse disabled:cursor-not-allowed disabled:opacity-30"
      >
        Import
      </button>
      <button
        @click="addScraper()"
        :disabled="isAddFileLocked"
        type="button"
        class="inline-flex items-center justify-center rounded-md border border-transparent bg-indigo-600 px-4 py-2 text-sm font-medium text-white shadow-sm hover:bg-indigo-700 focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2 disabled:animate-pulse disabled:cursor-not-allowed disabled:opacity-30 sm:w-auto"
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
                        selectedScrapers.length === files?.length) &&
                      selectedScrapers.length > 0
                    "
                    :indeterminate="indeterminate"
                    @change="
                      //@ts-ignore
                      selectedScrapers = $event.target?.checked
                        ? files.map((p) => p.id)
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
                v-for="scraper in files"
                :key="scraper.id"
                :class="[
                  selectedScrapers.includes(scraper.id) && 'bg-gray-50 ',
                  changedFilesPending.includes(scraper.id) &&
                    'bg-red-50 opacity-30',
                ]"
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
                    :disabled="isEnableLocked.includes(scraper.id)"
                    @click="handleToggle(scraper.id, !scraper.enabled)"
                    :class="[
                      isEnableLocked.includes(scraper.id) &&
                        ' cursor-wait opacity-30',
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
          <div class="bg-white px-4 py-12" v-if="files.length <= 0">
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
