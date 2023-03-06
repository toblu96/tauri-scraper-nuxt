<template>
  <div class="h-full bg-gray-100 py-8 px-6">
    <div class="space-y-6 bg-white px-4 py-5 shadow sm:rounded-lg sm:p-6">
      <div class="sticky top-0 bg-red-300">
        <PanelLogsFilter
          @export-log-data="exportFilteredLogLines"
          @filter-param-update="updateQueryFilter($event)"
          :pending="pending"
          :exporting="exporting"
        />
      </div>

      <!-- error -->
      <div class="rounded-md bg-red-50 p-4" v-if="error">
        <div class="flex">
          <div class="flex-shrink-0">
            <XCircleIcon class="h-5 w-5 text-red-400" aria-hidden="true" />
          </div>
          <div class="ml-3">
            <h3 class="text-sm font-medium text-red-800">
              {{ error }}
            </h3>
            <div class="mt-2 text-sm text-red-700">
              <ul role="list" class="list-disc space-y-1 pl-5">
                <li>{{ error?.data }}</li>
              </ul>
            </div>
          </div>
        </div>
      </div>

      <!-- skeleton loader -->
      <div v-if="pending" class="animate-pulse space-y-2">
        <div
          class="mt-16 h-6 w-full border-l-4 border-gray-300 bg-gray-100"
        ></div>
        <div class="h-6 w-full border-l-4 border-gray-300 bg-gray-100"></div>
      </div>

      <!-- data -->
      <div v-if="logData && !pending" class="space-y-1">
        <div class="mb-4 flex justify-between">
          <div>
            <span v-show="logData.length == 0"
              >Update your search params..</span
            >
          </div>
          <span
            class="inline-flex items-center rounded-md bg-gray-100 px-2.5 py-0.5 text-sm font-medium text-gray-800"
            >{{ logData.length }} log entries found</span
          >
        </div>
        <div
          v-for="log in logData"
          :key="log.time"
          :class="[
            log.level == 'DEBUG'
              ? 'border-indigo-300 bg-indigo-50 hover:bg-indigo-100'
              : '',
            log.level == 'TRACE'
              ? 'border-sky-300 bg-sky-50 hover:bg-sky-100'
              : '',
            log.level == 'INFO'
              ? 'border-gray-300 bg-gray-50 hover:bg-gray-100'
              : '',
            log.level == 'WARN'
              ? 'border-orange-300 bg-orange-50 hover:bg-orange-100'
              : '',
            log.level == 'ERROR'
              ? 'border-red-300 bg-red-50 hover:bg-red-100'
              : '',
          ]"
          class="border-l-4 border-gray-300 bg-gray-50 pl-2 hover:bg-gray-100 md:flex md:space-x-4"
        >
          <div class="flex-none md:w-40">
            <time :datetime="new Date(log.time).toLocaleString()">{{
              new Date(log.time).toLocaleString() || "??"
            }}</time>
          </div>
          <div class="grow">
            {{ log.message }}
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { XCircleIcon } from "@heroicons/vue/20/solid";
import { save } from "@tauri-apps/api/dialog";
import { writeTextFile } from "@tauri-apps/api/fs";

interface ILogMessage {
  time: string;
  message: string;
  level: string;
}

const pending = ref(true);
const exporting = ref(false);
const queryFilter = ref("");
const logData = ref<ILogMessage[] | undefined>(undefined);
const eventSource = ref(
  new EventSource(`http://localhost:8000/api/logs/sse${queryFilter.value}`)
);
// TODO: implement better error handling
let { error } = await useFetch<ILogMessage[]>(
  () => `http://localhost:8000/api/logs${queryFilter.value}`
);

// handle query filter updates
const updateQueryFilter = async (data: any) => {
  pending.value = true;

  // concat query filter
  let filter = `?message=${data.searchString}`;

  if (data.level && data.level != "") {
    filter += `&level=${data.level}`;
  }

  if (data.startDate && data.endDate) {
    filter += `&start_date=${encodeURIComponent(
      data.startDate
    )}&end_date=${encodeURIComponent(data.endDate)}`;
  }

  queryFilter.value = filter;

  // refresh eventsource filter params
  refreshEventSource();
};

// subscribe to active files from sse backend
const refreshEventSource = () => {
  eventSource.value.close();
  let tmpEventSource = new EventSource(
    `http://localhost:8000/api/logs/sse${queryFilter.value}`
  );
  // reset pending state in case of error (no log files found)
  tmpEventSource.onopen = function (event) {
    pending.value = false;
  };
  tmpEventSource.onmessage = function (event) {
    try {
      logData.value = JSON.parse(event.data);
    } catch (error) {
      console.error(`Could not update log lines: ${error}`);
    }
  };

  eventSource.value = tmpEventSource;
};

// export filtered logs
const exportFilteredLogLines = async () => {
  exporting.value = true;
  const filePath = await save({
    filters: [
      {
        name: "Application logs",
        extensions: ["json"],
      },
    ],
    defaultPath: `filtered-application-logs-${new Date().toLocaleDateString()}`,
  });
  // only save if user did not cancelled the selection
  if (filePath) {
    await writeTextFile(filePath, JSON.stringify(logData.value));
  }

  exporting.value = false;
};

// close eventsource on page leave
onUnmounted(() => {
  eventSource.value.close();
});
</script>
