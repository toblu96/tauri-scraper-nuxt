<template>
  <div class="h-full bg-gray-100">
    <div class="mx-auto h-full max-w-7xl overflow-auto py-12 sm:px-6 lg:px-8">
      <div class="mx-auto max-w-3xl">
        <div class="bg-white shadow sm:rounded-lg">
          <div class="px-4 py-5 sm:p-6">
            <h3 class="text-lg font-medium leading-6 text-gray-900">
              Seems like your backend is not running..
            </h3>
            <div class="mt-2 max-w-2xl text-sm text-gray-500">
              <p>
                For this tool to work properly, you need to run the backend web
                server which monitores the files and stores its states.
              </p>
              <div class="mt-8">
                <!-- information panel -->
                <div class="rounded-md bg-blue-50 p-4">
                  <div class="flex">
                    <div class="flex-shrink-0">
                      <InformationCircleIcon
                        class="h-5 w-5 text-blue-400"
                        aria-hidden="true"
                      />
                    </div>
                    <div class="ml-3 flex-1 md:flex md:justify-between">
                      <p class="text-sm text-blue-700">
                        If this is a fresh installation, you need to first add
                        the backend server as windows task.
                      </p>
                    </div>
                  </div>
                </div>

                <ol class="mt-4 list-inside list-decimal space-y-4">
                  <li>
                    Check current file location of server executable.
                    <div class="mt-2">
                      <code class="rounded-md bg-gray-100 px-2 py-1"
                        >C:\Program Files\EH File Version Monitor</code
                      >
                    </div>
                  </li>
                  <li>
                    Open Windows task scheduler and check if service "FC Version
                    Monitor Backend" exists.
                    <div class="mt-2">
                      <button
                        @click="openTaskScheduler"
                        class="font-medium text-indigo-600 hover:text-indigo-500"
                      >
                        Open task scheduler
                        <span aria-hidden="true"> &rarr;</span>
                      </button>
                    </div>
                  </li>
                  <li>
                    If service does not exist, add it as "simple task" with the
                    following configuration:
                    <div class="mt-2">
                      <pre
                        class="whitespace-pre-line rounded-md bg-gray-100 px-4 py-3"
                      >
                       <b>Create task</b>
                        name:           EH File Version Monitor Backend
                        description:    Backend service for FC monitoring application

                       <b>Trigger</b>
                        On machine start
                        
                       <b>Action</b>
                        Start application

                       <b>Application</b>
                        programm:   <i>Add application path from above</i> 
                        argument: "-s"

                        Important: Run application even if no user is logged in! To do so, go back to task settings.
                      </pre>
                    </div>
                  </li>
                  <li>
                    Start service manually and check if its online.
                    <div class="mt-2">
                      <button
                        @click="open('http://localhost:8000')"
                        class="font-medium text-indigo-600 hover:text-indigo-500"
                      >
                        Check OpenAPI docs
                        <span aria-hidden="true"> &rarr;</span>
                      </button>
                    </div>
                  </li>
                </ol>
              </div>
            </div>
          </div>
        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { InformationCircleIcon } from "@heroicons/vue/20/solid";
import { Command, open } from "@tauri-apps/api/shell";
definePageMeta({
  layout: false,
});

const openTaskScheduler = async () => {
  const output = await new Command("open-task-scheduler").execute();
  console.log(output);
};
</script>

<style>
html {
  @apply h-full overflow-hidden bg-gray-100;
}

body {
  @apply h-full;
}

#__nuxt {
  @apply h-full overflow-hidden;
}
</style>
