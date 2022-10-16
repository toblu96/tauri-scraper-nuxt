import { defineStore, acceptHMRUpdate } from 'pinia'

interface State {
  fileScrapers: Scraper[]
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
    fileScrapers: []
  }),
  actions: {
    async init() {
      console.log("init scraper store")
      const data = await tauriStore.get("settings-file-scrapers")
      this.fileScrapers = data || []
    },
    async tauriSave() {
      console.log("saved to tauri")
      await tauriStore.set("settings-file-scrapers", this.fileScrapers)
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
  },
  getters: {
    scraperList: state => state.fileScrapers,
  },
})

// enable hot module replacement
if (import.meta.hot) {
  import.meta.hot.accept(acceptHMRUpdate(useScraperStore, import.meta.hot))
}