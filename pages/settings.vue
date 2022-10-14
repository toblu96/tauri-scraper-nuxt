<script setup>
import { enable, disable, isEnabled } from "tauri-plugin-autostart-api";
import {
  Switch,
  SwitchDescription,
  SwitchGroup,
  SwitchLabel,
} from "@headlessui/vue";

const store = useTauriStore();
const autostartEnabled = ref(await store.get("settings-autostartEnabled"));

//autostart functionality
watchEffect(async () => {
  await store.set("settings-autostartEnabled", autostartEnabled.value);

  // only change autostart state if required
  if ((await isEnabled()) == autostartEnabled.value) return;

  autostartEnabled.value ? await enable() : await disable();

  console.log(
    `Autostart changed to: ${
      autostartEnabled.value
    }, store: ${await isEnabled()}`
  );
});
</script>

<template>
  <div class="space-y-6 divide-y divide-gray-200 pt-8">
    <div class="px-6">
      <div>
        <h2 class="text-lg font-medium leading-6 text-gray-900">
          Application Settings
        </h2>
        <p class="mt-1 text-sm text-gray-500">
          These settings will be stored persistent on the machine and be still
          available after restart.
        </p>
      </div>
      <ul role="list" class="mt-2 divide-y divide-gray-200">
        <SwitchGroup as="li" class="flex items-center justify-between py-4">
          <div class="flex flex-col">
            <SwitchLabel
              as="p"
              class="text-sm font-medium text-gray-900"
              passive
              >Autostart App</SwitchLabel
            >
            <SwitchDescription class="text-sm text-gray-500"
              >If enabled, this application will start on each server restart
              automatically.</SwitchDescription
            >
          </div>
          <Switch
            v-model="autostartEnabled"
            :class="[
              autostartEnabled ? 'bg-indigo-500' : 'bg-gray-200',
              'relative ml-4 inline-flex h-6 w-11 flex-shrink-0 cursor-pointer rounded-full border-2 border-transparent transition-colors duration-200 ease-in-out focus:outline-none focus:ring-2 focus:ring-indigo-500 focus:ring-offset-2',
            ]"
          >
            <span
              aria-hidden="true"
              :class="[
                autostartEnabled ? 'translate-x-5' : 'translate-x-0',
                'inline-block h-5 w-5 transform rounded-full bg-white shadow ring-0 transition duration-200 ease-in-out',
              ]"
            />
          </Switch>
        </SwitchGroup>
      </ul>
    </div>
    <section class="space-y-4 p-6">
      <h2 class="text-lg font-medium leading-6 text-gray-900">Danger Zone</h2>

      <ActionPanelSettingsDangerZone />
    </section>
  </div>
</template>
