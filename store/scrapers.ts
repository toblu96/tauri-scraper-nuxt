// store/filters.js
import { defineStore } from 'pinia'

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

export const useScrapersStore = defineStore('scraper-store', {
  state: (): State => ({
    fileScrapers: [
      { "id": "82e2ea3c-d0b7-4b06-b2c6-f5018009bae6", "name": "SC1", "enabled": true, "path": "C:\Users\i40010702\Desktop\Neues Textdokument.txt" },
      { "id": "070be24a-5dca-4081-ecc5-b82f63ee4368", "name": "SC2", "enabled": false, "path": "C:\Users\i40010702\Desktop\wach_imed.txt" }
    ]
  }),
  actions: {
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