<script setup lang="ts">
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
// subscribe to active files from sse backend
const files = ref<IFile[]>([]);
let eventSource = new EventSource("http://localhost:8000/api/files/sse");
eventSource.onmessage = function (event) {
  try {
    let unsortedFiles: IFile[] = JSON.parse(event.data);
    files.value = unsortedFiles.sort((a, b) => {
      if (a.path < b.path) return -1;
      if (a.path > b.path) return 1;
      return 0;
    });
  } catch (error) {
    console.error(`Could not update files: ${error}`);
  }
};
// close eventsource on page leave
onUnmounted(() => {
  eventSource.close();
});

async function handleToggle(id: string, state: boolean) {
  let res = await useFetch(`http://localhost:8000/api/files/${id}`, {
    method: "PATCH",
    body: {
      enabled: state,
    },
  });
  if (res.error.value) {
    console.error(res.error.value);
  }
}
</script>

<template>
  <div class="px-6 pt-8">
    <TableScrapers :scrapers="files" @toggleEnableState="handleToggle" />
  </div>
</template>
