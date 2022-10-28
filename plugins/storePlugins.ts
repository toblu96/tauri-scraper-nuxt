import { PiniaPluginContext } from 'pinia'
import { debounce } from 'ts-debounce'
import { PiniaDebounce } from '@pinia/plugin-debounce'
import { useScraperStore } from "~~/stores/scrapers";

async function PiniaAutosaveToTauri({ store }: PiniaPluginContext) {
    const scraperStore = useScraperStore();
    await scraperStore.init();
    console.log("init from store plugin")
    store.$subscribe((mutation) => {
        // mutation.events is only available on dev!
        scraperStore.tauriSave();
    })

    // Note this has to be typed if you are using TS
    return { creationTime: new Date() }
}

export default defineNuxtPlugin(({ $pinia }) => {
    $pinia.use(PiniaDebounce(debounce))
    $pinia.use(PiniaAutosaveToTauri)
})