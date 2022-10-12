<script setup>
import { invoke } from '@tauri-apps/api'
import { Store } from 'tauri-plugin-store-api';

const message = ref("")
const storeVal = ref(0)

async function greet(event) {
  message.value = await invoke("greet", { name: 'toubi' })
}

async function createStore(event) {
  const store = new Store('settings.dat');
  await store.get("data")
  await store.set('some-key', { value: 5 });
  storeVal.value = await store.get('some-key');
}


</script>
<template>
  <div>
    <button @click="greet()">Greet</button>
    <button @click="createStore()">Create Store</button>
    <p>{{message}}</p>
    {{storeVal}}
    <NuxtWelcome />
  </div>
</template>
