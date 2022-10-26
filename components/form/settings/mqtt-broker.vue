<script setup>
import { useScraperStore } from "~~/stores/scrapers";
import { Switch } from "@headlessui/vue";

const store = useScraperStore();
const broker = store.mqttBroker;

const secureBroker = ref(false);
secureBroker.value = broker.protocol === "mqtts://" ? true : false;

// trigger broker reconnection on settings changed
watch(
  () => broker,
  () => {
    store.reconnectMQTTBroker();
  },
  {
    deep: true,
  }
);
</script>
<template>
  <form class="space-y-6" action="#" method="POST">
    <div class="bg-white px-4 py-5 shadow sm:rounded-lg sm:p-6">
      <div class="md:grid md:grid-cols-3 md:gap-6">
        <div class="relative w-full md:col-span-1">
          <div>
            <h3 class="text-lg font-medium leading-6 text-gray-900">
              MQTT Broker
            </h3>
            <p class="mt-1 text-sm text-gray-500">
              Specify broker settings for sending file version data.
            </p>
          </div>
          <span
            v-if="store.mqttBrokerState.connected"
            class="absolute bottom-0 inline-flex items-center rounded-md bg-green-100 px-2.5 py-0.5 text-sm font-medium text-green-800"
          >
            <svg
              class="-ml-0.5 mr-1.5 h-2 w-2 animate-pulse text-green-400"
              fill="currentColor"
              viewBox="0 0 8 8"
            >
              <circle cx="4" cy="4" r="3" />
            </svg>
            Connected
          </span>
          <span
            v-else
            class="absolute bottom-0 inline-flex items-center rounded-md bg-red-100 px-2.5 py-0.5 text-sm font-medium text-red-800"
          >
            {{ store.mqttBrokerState.description || "Connection Error" }}
          </span>
        </div>
        <div class="mt-5 md:col-span-2 md:mt-0">
          <div class="grid grid-cols-6 gap-6">
            <div class="col-span-6 sm:col-span-4">
              <label
                for="broker-client-id"
                class="block text-sm font-medium text-gray-700"
                >Client ID</label
              >
              <input
                type="text"
                v-model="broker.clientId"
                name="broker-client-id"
                id="broker-client-id"
                class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
              />
            </div>

            <div class="col-span-6 flex flex-col justify-between sm:col-span-2">
              <label
                for="broker-secure"
                class="block text-sm font-medium text-gray-700"
                >SSL/TLS</label
              >
              <div
                class="place-self-stretch"
                title="Load certificates from server"
              >
                <Switch
                  id="broker-secure"
                  v-model="secureBroker"
                  @click="store.toggleBrokerSecurity()"
                  :class="[
                    secureBroker ? 'bg-indigo-500' : 'bg-gray-200',
                    'relative  inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2',
                  ]"
                >
                  <span
                    aria-hidden="true"
                    :class="[
                      secureBroker ? 'translate-x-5' : 'translate-x-0',
                      'inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
                    ]"
                  />
                </Switch>
              </div>
            </div>

            <div class="col-span-4">
              <div>
                <label
                  for="broker-host"
                  class="block text-sm font-medium text-gray-700"
                  >Host</label
                >
                <div class="mt-1 flex rounded-md shadow-sm">
                  <span
                    class="inline-flex items-center rounded-l-md border border-r-0 border-gray-300 bg-gray-50 px-3 text-gray-500 sm:text-sm"
                    >{{ broker.protocol }}</span
                  >
                  <input
                    type="text"
                    name="broker-host"
                    id="broker-host"
                    class="block w-full min-w-0 flex-1 rounded-none rounded-r-md border-gray-300 px-3 py-2 focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
                    placeholder="localhost"
                    v-model="broker.host"
                  />
                </div>
              </div>
            </div>
            <div class="col-span-2">
              <div>
                <label
                  for="broker-port"
                  class="block text-sm font-medium text-gray-700"
                  >Port</label
                >
                <div class="mt-1">
                  <input
                    type="number"
                    name="broker-port"
                    id="broker-port"
                    class="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
                    placeholder="1883"
                    v-model="broker.port"
                  />
                </div>
              </div>
            </div>
            <div class="col-span-3">
              <div>
                <label
                  for="broker-username"
                  class="block text-sm font-medium text-gray-700"
                  >Username</label
                >
                <div class="mt-1">
                  <input
                    type="text"
                    name="broker-username"
                    id="broker-username"
                    class="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
                    placeholder="admin"
                    v-model="broker.username"
                  />
                </div>
              </div>
            </div>
            <div class="col-span-3">
              <div>
                <label
                  for="broker-password"
                  class="block text-sm font-medium text-gray-700"
                  >Password</label
                >
                <div class="mt-1">
                  <input
                    type="password"
                    name="broker-password"
                    id="broker-password"
                    class="block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
                    placeholder="1234"
                    v-model="broker.password"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </form>
</template>
