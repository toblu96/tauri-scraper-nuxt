import { Store } from "tauri-plugin-store-api";
const store = new Store("settings.dat");

export const useStore = () => {
    return store
}