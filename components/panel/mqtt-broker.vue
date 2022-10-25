<template>
  <div class="overflow-hidden bg-white shadow sm:rounded-md">
    <div class="px-4 py-4 sm:px-6 md:flex md:items-center md:justify-between">
      <div class="min-w-0 flex-1">
        <h2
          class="text-lg font-bold leading-7 text-gray-900 sm:truncate sm:text-xl sm:tracking-tight"
        >
          MQTT Broker
        </h2>
        <div
          class="mt-1 flex flex-col sm:mt-0 sm:flex-row sm:flex-wrap sm:space-x-6"
        >
          <div class="mt-2 flex items-center text-sm text-gray-500">
            <ShieldExclamationIcon
              v-if="store.mqttBroker.protocol === 'mqtt://'"
              class="mr-1.5 h-5 w-5 flex-shrink-0 text-gray-400"
              aria-hidden="true"
            />
            <ShieldCheckIcon
              v-else
              class="mr-1.5 h-5 w-5 flex-shrink-0 text-gray-400"
              aria-hidden="true"
            />
            {{ store.mqttBroker.host }} :
            {{ store.mqttBroker.port }}
          </div>
          <div class="mt-2 flex items-center text-sm text-gray-500">
            <IdentificationIcon
              class="mr-1.5 h-5 w-5 flex-shrink-0 text-gray-400"
              aria-hidden="true"
            />
            {{ store.mqttBroker.clientId }}
          </div>
        </div>
      </div>
      <div class="mt-5 flex md:mt-0 md:ml-4">
        <span
          v-if="store.mqttBrokerState.connected"
          class="inline-flex items-center rounded-md bg-green-100 px-2.5 py-0.5 text-sm font-medium text-green-800 lg:py-2 lg:px-5"
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
          class="inline-flex items-center rounded-md bg-red-100 px-2.5 py-0.5 text-sm font-medium text-red-800 lg:py-2 lg:px-5"
        >
          {{ store.mqttBrokerState.description || "Connection Error" }}
        </span>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import {
  BriefcaseIcon,
  CalendarIcon,
  CheckIcon,
  ChevronDownIcon,
  CurrencyDollarIcon,
  LinkIcon,
  MapPinIcon,
  PencilIcon,
  LockClosedIcon,
  ShieldCheckIcon,
  ShieldExclamationIcon,
  IdentificationIcon,
} from "@heroicons/vue/20/solid";

import { useScraperStore } from "~~/stores/scrapers";
const store = useScraperStore();
</script>
