import { Store } from "tauri-plugin-store-api";
const store = new Store("settings.dat");
console.log("composable once")

export const useStore = () => {
    console.log("composable")
    return store
}