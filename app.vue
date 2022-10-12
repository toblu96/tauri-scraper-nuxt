<script setup>
import { invoke } from "@tauri-apps/api";
import { Store } from "tauri-plugin-store-api";

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
  await store.set("some-key", { value: 3 });
}

async function getStoreValue(event) {
  storeVal.value = await store.get("some-key");
}
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
    <NuxtWelcome />
  </div>
</template>
