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
const activeFiles = ref<IFile[]>([]);
let eventSource = new EventSource("http://localhost:8000/api/files/sse");
eventSource.onmessage = function (event) {
  try {
    let files: IFile[] = JSON.parse(event.data);
    activeFiles.value = files.filter((file) => file.enabled);
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
  <div class="space-y-6 px-6 pt-8">
    <PanelMqttBroker />
    <ListScraperInfo :scrapers="activeFiles" />
  </div>
</template>
