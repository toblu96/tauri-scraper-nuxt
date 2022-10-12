<script setup>
import { invoke } from "@tauri-apps/api";
import { Store } from "tauri-plugin-store-api";
import { watch, watchImmediate } from "tauri-plugin-fs-watch-api";

const message = ref("");
const storeVal = ref(0);
const store = new Store("settings.dat");

async function greet(event) {
  message.value = await invoke("greet", { name: "toubi" });
}

async function createStore(event) {
  await store.get("data");
  await store.set("some-key", { value: 5 });
  storeVal.value = await store.get("some-key");
}

async function setStoreValue(event) {
  await store.clear();
  // await store.set("some-key", { value: 3 });
}

async function getStoreValue(event) {
  storeVal.value = await store.entries();
}

// can also watch an array of paths
const stopWatching = await watch(
  "C:\\Users\\i40010702\\Desktop\\Neues Textdokument.txt",
  { recursive: true },
  (event) => {
    const { type, payload } = event;
    console.log(`Watchs 'Neues Textdokument.txt': ${type} - ${payload}`);
  }
);

const stopRawWatcher = await watchImmediate(
  ["C:\\Users\\i40010702\\Desktop\\wach_imed.txt"],
  {},
  (event) => {
    const { path, operation, cookie } = event;
    console.log(
      `Watch imed 'wach_imed.txt': ${path} - ${operation} - ${cookie}`
    );
  }
);

onBeforeUnmount(async () => {
  await stopWatching();
  await stopRawWatcher();
});
</script>
<template>
  <div>
    <Stats />
    <button @click="greet()">Greet</button>
    <button @click="createStore()">Create Store</button>
    <div class="m-4 space-x-4 border-l bg-green-50">
      <button @click="setStoreValue()">set Store</button>
      <button @click="getStoreValue()">get Store</button>
    </div>
    <p>{{ message }}</p>
    {{ storeVal }}
  </div>
</template>
