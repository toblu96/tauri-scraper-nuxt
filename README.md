<h1 align="center">EH Version Scraper</h1>
Checks files and services for version change on a windows machine.

<br/>

# Application Installation on Windows Server 2019

## Global WebView2 setup

Globally install WebView2. Tauri uses it to run and display the frontend code. Download and install it from [here](https://developer.microsoft.com/en-us/microsoft-edge/webview2/#download-section). Install it for all users.

## Application Installation

Under construction...

## Uninstall application

Under construction...

# Project Setup

> ⚠ Please note that Windows .msi installers can only be created on Windows as cross-compilation doesn't work yet. Track progress [here](https://tauri.app/v1/guides/building/cross-platform).

## Tauri Prerequisites

Check [Tauri prerequisites for Windows](https://tauri.app/v1/guides/getting-started/prerequisites#setting-up-windows).

This will guide you through the necessary prerequisites installations.

1. Microsoft Visual Studio C++ Build Tools
2. WebView2 (Preinstalled in Windows 11)
3. Rust

## Nuxt initial setup

Required prerequisites for Nuxt. Check detailed instructions [here](https://v3.nuxtjs.org/getting-started/installation#prerequisites).

1. Node.js
2. VS Code Volar Extension

Install node dependencies.

```bash
npm install
```

## Development Application

Start the application on your local machine with HMR feature enabled for frontend and backend.

```bash
npm run tauri dev
```

## Production

Build the application for production:

```bash
npm run tauri build
```
