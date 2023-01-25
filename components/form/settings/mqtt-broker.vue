<script setup lang="ts">
import { Switch } from "@headlessui/vue";
import { QuestionMarkCircleIcon } from "@heroicons/vue/20/solid";
import { debounce } from "ts-debounce";

enum MqttProtocol {
  mqtt = "mqtt://",
  mqtts = "mqtts://",
}

interface IMqttBrokerSettings {
  client_id: string;
  host: string;
  port: number;
  protocol: MqttProtocol;
  username: string;
  password: string;
  device_id: string;
  device_group: string;
  state: string;
  connected: boolean;
}

const blockEventSourceUpdates = ref(false);
const isBrokerReconnecting = ref(false);

// init broker settings with live values - prevents watcher to send update to server on init
const brokerInitData = await useFetch(
  "http://localhost:8000/api/settings/broker"
);
const brokerSettings = ref<IMqttBrokerSettings>(
  brokerInitData.data.value as IMqttBrokerSettings
);
const isTLSActive = ref(brokerSettings.value.protocol === "mqtts://");

// subscribe to active files from sse backend
let eventSource = new EventSource("http://localhost:8000/api/settings/sse");
eventSource.onmessage = function (event) {
  // do not overrride pending changes
  if (blockEventSourceUpdates.value) return;

  try {
    let brokerData: IMqttBrokerSettings = JSON.parse(event.data);

    // only update reactive values on change, otherwise watcher will create an infinite loop
    if (brokerSettings.value.client_id != brokerData.client_id)
      brokerSettings.value.client_id = brokerData.client_id;
    if (brokerSettings.value.device_group != brokerData.device_group)
      brokerSettings.value.device_group = brokerData.device_group;
    if (brokerSettings.value.device_id != brokerData.device_id)
      brokerSettings.value.device_id = brokerData.device_id;
    if (brokerSettings.value.host != brokerData.host)
      brokerSettings.value.host = brokerData.host;
    if (brokerSettings.value.port != brokerData.port)
      brokerSettings.value.port = brokerData.port;
    if (brokerSettings.value.password != brokerData.password)
      brokerSettings.value.password = brokerData.password;
    if (brokerSettings.value.protocol != brokerData.protocol)
      brokerSettings.value.protocol = brokerData.protocol;
    if (brokerSettings.value.username != brokerData.username)
      brokerSettings.value.username = brokerData.username;
    if (brokerSettings.value.connected != brokerData.connected)
      brokerSettings.value.connected = brokerData.connected;
    if (brokerSettings.value.state != brokerData.state)
      brokerSettings.value.state = brokerData.state;

    if (isTLSActive.value != (brokerSettings.value.protocol === "mqtts://"))
      isTLSActive.value = brokerSettings.value.protocol === "mqtts://";
  } catch (error) {
    console.error(`Could not update broker settings: ${error}`);
  }
  isBrokerReconnecting.value = false;
};
// close eventsource on page leave
onUnmounted(() => {
  eventSource.close();
});

// trigger broker settings change
const updateBrokerSettings = debounce(async () => {
  let res = await useFetch("http://localhost:8000/api/settings/broker", {
    method: "PATCH",
    body: {
      client_id: brokerSettings.value.client_id,
      device_group: brokerSettings.value.device_group,
      device_id: brokerSettings.value.device_id,
      host: brokerSettings.value.host,
      password: brokerSettings.value.password,
      port: brokerSettings.value.port,
      username: brokerSettings.value.username,
      protocol: isTLSActive.value ? "mqtts://" : "mqtt://",
    },
  });
  if (res.error.value) {
    console.error(res.error.value);
  }
  blockEventSourceUpdates.value = false;
}, 1000);
watch(
  () => [
    isTLSActive.value,
    brokerSettings.value.client_id,
    brokerSettings.value.device_group,
    brokerSettings.value.device_id,
    brokerSettings.value.host,
    brokerSettings.value.password,
    brokerSettings.value.port,
    brokerSettings.value.username,
  ],
  () => {
    blockEventSourceUpdates.value = true;
    isBrokerReconnecting.value = true;
    updateBrokerSettings();
  }
);
</script>
<template>
  <form class="space-y-6" action="#" method="POST">
    <div
      class="space-y-6 divide-y divide-gray-200 bg-white px-4 py-5 shadow sm:rounded-lg sm:p-6"
    >
      <!-- Broker config -->
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
            v-if="isBrokerReconnecting"
            class="absolute bottom-0 inline-flex items-center rounded-md bg-yellow-100 px-2.5 py-0.5 text-sm font-medium text-yellow-800"
          >
            <svg
              class="-ml-0.5 mr-1.5 h-2 w-2 animate-pulse text-yellow-400"
              fill="currentColor"
              viewBox="0 0 8 8"
            >
              <circle cx="4" cy="4" r="3" />
            </svg>
            Reconnecting..
          </span>
          <span
            v-else-if="brokerSettings.connected"
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
            {{ brokerSettings.state || "Connection Error" }}
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
                v-model="brokerSettings.client_id"
                name="broker-client-id"
                id="broker-client-id"
                class="mt-1 block w-full rounded-md border-gray-300 shadow-sm focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
              />
            </div>

            <div
              class="col-span-6 flex flex-col justify-between sm:col-span-2"
              title="Load certificates from server"
            >
              <div class="flex">
                <label
                  for="broker-secure"
                  class="block text-sm font-medium text-gray-700"
                  >TLS/SSL</label
                >
                <span class="ml-4 text-sm text-gray-500" id="broker-secure"
                  ><QuestionMarkCircleIcon
                    class="h-5 w-5 text-gray-400"
                    aria-hidden="true"
                /></span>
              </div>
              <div class="place-self-stretch">
                <Switch
                  id="broker-secure"
                  v-model="isTLSActive"
                  :class="[
                    isTLSActive ? 'bg-indigo-500' : 'bg-gray-200',
                    'relative  inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2',
                  ]"
                >
                  <span
                    aria-hidden="true"
                    :class="[
                      isTLSActive ? 'translate-x-5' : 'translate-x-0',
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
                    >{{ brokerSettings.protocol }}</span
                  >
                  <input
                    type="text"
                    name="broker-host"
                    id="broker-host"
                    class="block w-full min-w-0 flex-1 rounded-none rounded-r-md border-gray-300 px-3 py-2 focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
                    placeholder="localhost"
                    v-model="brokerSettings.host"
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
                    v-model="brokerSettings.port"
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
                    v-model="brokerSettings.username"
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
                    v-model="brokerSettings.password"
                  />
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
      <!-- Solman config -->
      <div class="pt-6 md:grid md:grid-cols-3 md:gap-6">
        <div class="relative w-full md:col-span-1">
          <div>
            <h3 class="text-base font-medium leading-6 text-gray-900">
              Solman Configuration
            </h3>
            <p class="mt-1 text-sm text-gray-500">
              Specify solman device settings.
            </p>
          </div>
        </div>
        <div class="mt-5 md:col-span-2 md:mt-0">
          <div class="grid grid-cols-6 gap-6">
            <div class="col-span-3">
              <div>
                <label
                  for="solman-device-id"
                  class="block text-sm font-medium text-gray-700"
                  >Device Id</label
                >
                <div
                  class="relative mt-1 rounded-md shadow-sm"
                  title="Same value as Device Id in Solman Device."
                >
                  <input
                    type="text"
                    name="solman-device-id"
                    id="solman-device-id"
                    class="block w-full rounded-md border-gray-300 pr-10 focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
                    placeholder="FC_0103"
                    v-model="brokerSettings.device_id"
                  />
                  <div
                    class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3"
                  >
                    <QuestionMarkCircleIcon
                      class="h-5 w-5 text-gray-400"
                      aria-hidden="true"
                    />
                  </div>
                </div>
              </div>
            </div>
            <div class="col-span-3">
              <div>
                <label
                  for="solman-device-group"
                  class="block text-sm font-medium text-gray-700"
                  >Device Group</label
                >
                <div
                  class="relative mt-1 rounded-md shadow-sm"
                  title="Same value as Device Group in Solman Device."
                >
                  <input
                    type="text"
                    name="solman-device-group"
                    id="solman-device-group"
                    class="block w-full rounded-md border-gray-300 pr-10 focus:border-indigo-500 focus:ring-indigo-500 sm:text-sm"
                    placeholder="autogroup_Monitor"
                    v-model="brokerSettings.device_group"
                  />
                  <div
                    class="pointer-events-none absolute inset-y-0 right-0 flex items-center pr-3"
                  >
                    <QuestionMarkCircleIcon
                      class="h-5 w-5 text-gray-400"
                      aria-hidden="true"
                    />
                  </div>
                </div>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </form>
</template>
