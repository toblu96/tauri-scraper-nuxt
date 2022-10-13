import { watch } from "tauri-plugin-fs-watch-api";

const store = useStore()

interface WatchConfig {
    id: number
    name: string
    enabled: boolean
    lastUpdate?: string
    path: string
}

interface ActiveWatcher {
    id: number
    stop(): Promise<void>
}

let watchConfig: WatchConfig[] = []
let activeWatcher: ActiveWatcher[] = []


/**
 * Lists all configured file watchers.
 * @returns List of all watch configs
 */
const listWatchers = () => {
    return watchConfig
}

/**
 * Removes a single file watcher by its id.
 * @param id Id of watcher to remove.
 */
const removeWatcher = async (id: number) => {
    await activeWatcher.find(watcher => watcher.id === id).stop()
}

/**
 * Creates a single file watcher.
 * @param watcher Watcher Configuration
 */
const addWatcher = async (watcher: WatchConfig) => {
    //TODO: add guards for params and check if it already exists
    activeWatcher.push({
        id: watcher.id,
        stop: await watch(
            watcher.path,
            { recursive: true },
            (event) => {
                const { type, payload } = event;
                if (type === 'Write')
                    console.log(`Watch ${watcher.path}: ${type} - ${payload}`);
                // emit tauri event to backend (handle fs in frontend?)
            }
        )
    })
}

const addConfig = async (watcher: WatchConfig) => {
    // check if data is valid
    // add config to local variable
    watchConfig.push(watcher)
    await store.set("settings-watch-paths", watchConfig)
    // enable watcher if config is enabled
}
const editConfig = async (watcher: WatchConfig) => {
    // check if data is valid
    // replace data
    // restart watcher if relevant config changed
}
const deleteConfig = async (id: number) => {
    // disable watcher
    // remove from local var
    watchConfig.filter(watchConfig => watchConfig.id != id)
}

export default defineNuxtPlugin(nuxtApp => {
    nuxtApp.hook("app:beforeMount", async () => {
        // load current configurations and activate them

        /*
        - Array of path to watch
        - enable / disabled
        - last update
 
        */
        const initStoreSettnigs: WatchConfig[] = [
            {
                id: 1234,
                name: "first path",
                enabled: true,
                path: "C:\\Users\\i40010702\\Desktop\\Neues Textdokument.txt"
            },
            {
                id: 1295,
                name: "first path",
                enabled: true,
                path: "C:\\Users\\i40010702\\Desktop\\wach_imed.txt"
            },
        ]
        await store.set("settings-watch-paths", initStoreSettnigs)

        watchConfig = await store.get("settings-watch-paths")

        // initialize watchers
        watchConfig.forEach((watcher, idx, watchers) => {
            if (watcher.enabled) {
                addWatcher(watcher)
            }
        })

    })

    return {
        provide: {
            listFileWatchers: listWatchers,
            enableFileWatchers: addWatcher,
            disableFileWatchers: removeWatcher,
            addFileWatcherConfig: addConfig,
            editFileWatcherConfig: editConfig,
            removeFileWatcherConfig: deleteConfig
        }
    }
})