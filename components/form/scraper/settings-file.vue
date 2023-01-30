<script setup lang="ts">
import { Switch } from "@headlessui/vue";
import { DocumentIcon } from "@heroicons/vue/24/solid";
import { open } from "@tauri-apps/api/dialog";
import { debounce } from "ts-debounce";

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

const route = useRoute();
const scraper = ref<IFile>();
const { data, refresh } = await useFetch<IFile[]>(
  `http://localhost:8000/api/files`
);
scraper.value = data.value?.find((scraper) => scraper.id === route.params.id);

// trigger file settings change
const isEnableLocked = ref(false);
const updateFileSettings = debounce(async () => {
  console.log("changed");
  let res = await useFetch(
    `http://localhost:8000/api/files/${route.params.id}`,
    {
      method: "PATCH",
      body: {
        enabled: scraper.value?.enabled,
        name: scraper.value?.name,
        path: scraper.value?.path,
      },
    }
  );
  if (res.error.value) {
    console.error(res.error.value);
  }
  await refresh();
  isEnableLocked.value = false;
}, 1000);
watch(
  () => [scraper.value?.path, scraper.value?.name, scraper.value?.enabled],
  (new_config, current_config) => {
    if (new_config[2] != current_config[2]) isEnableLocked.value = true;
    updateFileSettings();
  }
);

const changeFilePath = async () => {
  // Open a selection dialog for directories
  const selected = await open({
    directory: false,
    multiple: false,
  });
  if (selected === null) {
    // user cancelled the selection
    console.log("aborted file selection");
  } else {
    // user selected a single directory
    console.log(selected);
    if (scraper.value) scraper.value.path = selected as string;
  }
};

// get realtime status update of file state
const lastFileState = ref<string>("");
let eventSource = new EventSource("http://localhost:8000/api/files/sse");
eventSource.onmessage = function (event) {
  console.log("got one");
  try {
    let files: IFile[] = JSON.parse(event.data);
    lastFileState.value =
      files.filter((file) => file.id == route.params.id)[0].update_state ||
      "could not update state";
  } catch (error) {
    console.error(`Could not update files: ${error}`);
  }
};
// close eventsource on page leave
onUnmounted(() => {
  eventSource.close();
});
</script>
<template>
  <form v-if="scraper" class="space-y-6" action="#" method="POST">
    <div class="bg-white px-4 py-5 shadow sm:rounded-lg sm:p-6">
      <div class="md:grid md:grid-cols-3 md:gap-6">
        <div class="relative w-full md:col-span-1">
          <div>
            <h3 class="text-lg font-medium leading-6 text-gray-900">File</h3>
            <p class="mt-1 text-sm text-gray-500">
              Specify the file and content you want to scrape.
            </p>
          </div>
          <span
            class="absolute bottom-0 inline-flex items-center rounded-md px-2.5 py-0.5 text-sm font-medium"
            :class="
              lastFileState == 'Success'
                ? 'bg-green-100 text-green-800'
                : 'bg-red-100 text-red-800'
            "
          >
            {{ lastFileState }}
          </span>
        </div>
        <div class="mt-5 md:col-span-2 md:mt-0">
          <div class="grid grid-cols-6 gap-6">
            <div class="col-span-6 sm:col-span-4">
              <label
                for="scraper-name"
                class="block text-sm font-medium text-gray-700"
                >Scraper name</label
              >
              <input
                type="text"
                v-model="scraper.name"
                name="scraper-name"
                id="scraper-name"
                class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
              />
            </div>

            <div class="col-span-6 flex flex-col justify-between sm:col-span-2">
              <label
                for="scraper-enabled"
                class="block text-sm font-medium text-gray-700"
                >Enabled</label
              >
              <div class="place-self-stretch">
                <Switch
                  id="scraper-enabled"
                  :disabled="isEnableLocked"
                  v-model="scraper.enabled"
                  :class="[
                    isEnableLocked && ' cursor-wait opacity-30',
                    scraper.enabled ? 'bg-indigo-500' : 'bg-gray-200',
                    'relative  inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2',
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
              </div>
            </div>

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
                      v-model="scraper.path"
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
          </div>
        </div>
      </div>
    </div>
  </form>
  <div v-else>
    <p>Could not find file..</p>
  </div>
</template>
