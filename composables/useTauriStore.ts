import { Store } from "tauri-plugin-store-api";
import { getName } from "@tauri-apps/api/app";

const appName = await getName();
const store = new Store(`C:\\ProgramData\\Tauri\\${appName}\\settings.dat`);

export const useTauriStore = () => {
    return store
}