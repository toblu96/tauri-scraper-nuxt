import { defineStore, acceptHMRUpdate } from 'pinia'
import { listen } from '@tauri-apps/api/event'
import { invoke } from '@tauri-apps/api/tauri'

interface State {
  fileScrapers: Scraper[]
  mqttBrokerSettings: MqttBroker
  mqttBrokerState: MqttBrokerState
}

type Scraper = {
  id: string;
  name: string;
  enabled: boolean;
  lastUpdate?: string;
  path: string;
};
type ScraperProps = {
  name: string;
  enabled: boolean;
  lastUpdate?: string;
  path: string;
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
      // backend events
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
    },
    addFileScraper(scraper: ScraperProps) {
      this.fileScrapers.push({ id: generateUUID(), ...scraper })
    },
    removeFileScraper(id: string) {
      this.fileScrapers.splice(this.fileScrapers.findIndex((obj) => obj.id === id), 1)
    },
    toggleEnableState(id: string) {
      for (const scraper of this.fileScrapers) {
        if (scraper.id === id) {
          scraper.enable = !scraper.enable
        }
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
  },
})

// enable hot module replacement
if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useScraperStore, import.meta.hot))
}