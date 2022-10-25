import { defineStore, acceptHMRUpdate } from 'pinia'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/tauri'
import { message } from '@tauri-apps/api/dialog';
import { watch } from "tauri-plugin-fs-watch-api";

interface State {
  fileScrapers: Scraper[]
  mqttBrokerSettings: MqttBroker
  mqttBrokerState: MqttBrokerState
}

type Scraper = {
  id: string;
  name: string;
  enabled: boolean;
  lastUpdateUTC?: string; // timestamp UTC
  updateState?: string; // status of update - e.g. could not read | successful
  lastVersion?: string; // latest file version
  path: string;
  mqttTopic: string;
  stop(): Promise<void>
};
type ScraperProps = {
  name: string;
  enabled: boolean;
  lastUpdate?: string;
  path: string;
  mqttTopic: string;
};

enum MqttProtocol {
  mqtt = "mqtt://",
  mqtts = "mqtts://"
}

type MqttBroker = {
  clientId: string
  host: string
  port: number
  protocol: MqttProtocol
}

type MqttBrokerState = {
  connected: boolean
}

const tauriStore = useTauriStore();

const generateUUID = () => {
  let
    d = new Date().getTime(),
    d2 = (performance && performance.now && (performance.now() * 1000)) || 0;
  return 'xxxxxxxx-xxxx-4xxx-yxxx-xxxxxxxxxxxx'.replace(/[xy]/g, c => {
    let r = Math.random() * 16;
    if (d > 0) {
      r = (d + r) % 16 | 0;
      d = Math.floor(d / 16);
    } else {
      r = (d2 + r) % 16 | 0;
      d2 = Math.floor(d2 / 16);
    }
    return (c == 'x' ? r : (r & 0x7 | 0x8)).toString(16);
  });
};

const executeScraper = async (scraper: Scraper, brokerConnected: boolean) => {
  scraper.lastUpdateUTC = new Date().toISOString()
  // read file version 
  let version = await invoke("plugin:file-version|get_file_version", { path: scraper.path })
  if (version === 'Could not read version.') {
    scraper.updateState = `Could not read file version from file '${scraper.path}'.`
    return
  }
  console.log("get version: ", scraper.updateState)
  // save last update value to local store
  scraper.lastVersion = version as string
  // check if broker is connected
  if (!brokerConnected) {
    scraper.updateState = "MQTT Broker disconnected"
    return
  }

  // publish version number
  let clientErr = await invoke("plugin:mqtt-client|publish", {
    topic: scraper.mqttTopic,
    payload: {
      version: scraper.lastVersion || "no data"
    },
  });
  if (clientErr) {
    scraper.updateState = `Could not publish message: ${clientErr}`
  } else {
    scraper.updateState = "Successful"
  }
}

const registerFileWatcher = async (scraper: Scraper, brokerState: MqttBrokerState) => {
  console.log("start scraper with name ", scraper.name)
  try {
    scraper.stop = await watch(
      scraper.path,
      { recursive: true },
      async (event) => {
        const { type, payload } = event;
        if (["Create", "Write", "Chmod", "Remove", "Rename", "Rescan", "Error"].includes(type)) {
          console.log(`Watch ${scraper.name}: ${type} - ${payload}`);
          await executeScraper(scraper, brokerState.connected)
        }
      }
    )
  } catch (error) {
    message(`Could not activate file watcher: \n${error}`, { title: 'Tauri | Enable file scraper', type: 'warning' })
    scraper.enabled = false
  }

}

const unregisterFileWatcher = async (scraper: Scraper) => {
  console.log("stop scraper with name ", scraper.name)
  await scraper.stop()
}

export const useScraperStore = defineStore('scraper-store', {
  state: (): State => ({
    fileScrapers: [],
    mqttBrokerSettings: {
      clientId: "tauri-mqtt-client",
      host: "localhost",
      port: 1883,
      protocol: MqttProtocol.mqtt
    },
    mqttBrokerState: {
      connected: false
    }
  }),
  actions: {
    async init() {
      console.log("init scraper store")
      // initial scraper settings
      const data = await tauriStore.get("settings-file-scrapers")
      this.fileScrapers = data || []
      // initial broker settings
      const broker = await tauriStore.get("settings-file-mqtt-broker")
      this.mqttBrokerSettings = broker || {
        clientId: "tauri-mqtt-client",
        host: "localhost",
        port: 1883,
        protocol: MqttProtocol.mqtt
      }
      // connect broker
      await invoke("plugin:mqtt-client|connect", {
        clientId: this.mqttBroker.clientId,
        host: this.mqttBroker.host,
        port: this.mqttBroker.port,
      });
      // start enabled file watchers
      for (const scraper of this.fileScrapers) {
        if (scraper.enabled)
          await registerFileWatcher(scraper, this.mqttBrokerState)
      }
      // register tauri events
      await listen("plugin:mqtt//connected", (event) => {
        this.mqttBrokerState.connected = event.payload;
      });
    },
    async tauriSave(event) {
      console.log("saved to tauri", event)
      await tauriStore.set("settings-file-scrapers", this.fileScrapers)
      await tauriStore.set("settings-file-mqtt-broker", this.mqttBrokerSettings)
      // reconnect broker if settings change
      if (event.events.target.clientId) {
        await invoke("plugin:mqtt-client|connect", {
          clientId: this.mqttBroker.clientId,
          host: this.mqttBroker.host,
          port: this.mqttBroker.port,
        });
      }
      // renew watcher if settings change
      if (event.events.target.stop && ['path', 'name'].includes(event.events.key)) { // only reload on certain settings
        let scraper: Scraper = this.fileScrapers.filter(scraper => scraper.id === event.events.target.id)[0];
        if (scraper.enabled) {
          await unregisterFileWatcher(scraper)
          await registerFileWatcher(scraper, this.mqttBrokerState)
        }
      }
    },
    addFileScraper(scraper: ScraperProps) {
      this.fileScrapers.push({ id: generateUUID(), ...scraper })
    },
    removeFileScraper(id: string) {
      this.fileScrapers.splice(this.fileScrapers.findIndex((obj) => obj.id === id), 1)
    },
    async toggleEnableState(id: string) {
      for (const scraper of this.fileScrapers) {
        if (scraper.id === id) {
          scraper.enabled = !scraper.enabled
        }
      }

      // handle file watcher
      let scraper: Scraper = this.fileScrapers.filter(scraper => scraper.id === id)[0];
      if (scraper.enabled) {
        await registerFileWatcher(scraper, this.mqttBrokerState)
        await executeScraper(scraper, this.mqttBrokerState.enabled)
      } else {
        await unregisterFileWatcher(scraper)
      }

    },
    toggleBrokerSecurity() {
      console.log(this.mqttBrokerSettings.protocol)
      this.mqttBrokerSettings.protocol = this.mqttBrokerSettings.protocol === MqttProtocol.mqtt ? MqttProtocol.mqtts : MqttProtocol.mqtt
    }
  },
  getters: {
    scraperList: state => state.fileScrapers,
    mqttBroker: state => state.mqttBrokerSettings,
    // brokerStateConnected: state => state.mqttBrokerState.connected
    enabledFileScrapers: state => state.fileScrapers.filter(scraper => scraper.enabled)
  },
})

// enable hot module replacement
if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useScraperStore, import.meta.hot))
}