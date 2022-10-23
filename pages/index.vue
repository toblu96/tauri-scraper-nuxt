<script setup>
import { invoke } from "@tauri-apps/api";

const test = async () => {
  await invoke("plugin:mqtt-client|connect");
};
</script>
<template>
  <div>
    <p>Hello from Home</p>

    <button class="rounded bg-red-300 py-2 px-4" @click="test">init</button>
    <button
      class="rounded bg-red-300 py-2 px-4"
      @click="invoke('plugin:mqtt-client|publish')"
    >
      publish
    </button>
  </div>
</template>
