import { Store } from "tauri-plugin-store-api";

export const useStore = () => {
    console.log("composable")
    return new Store("settings.dat");
}