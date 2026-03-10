# 🚀 OpenQuest Hub

**OpenQuest Hub** is a powerful, modern, and cross-platform desktop application designed for seamless management of Meta Quest (and other Android-based) VR headsets. Built with **Tauri**, **Rust**, and **React**, it provides a premium user experience for developers and enthusiasts alike.

![OpenQuest Hub - Premium ADB Manager](https://via.placeholder.com/1000x400?text=OpenQuest+Hub+UI+Interface)

---

## ✨ Key Features

- **📱 Device Management:** Real-time polling of connected devices with status indicators (Online, Unauthorized, Offline).
- **📦 App Explorer:** Browse, uninstall, and launch installed applications. Sideload APKs with a simple drag-and-drop.
- **📁 File Explorer:** Full access to device storage (`/sdcard`). Download files to your PC with real-time transfer progress.
- **📟 Powerful Logcat:** High-performance log streaming with custom argument support (e.g., `-s unity`), filtering, and pausing.
- **⚙️ Deep Configuration:** Persistent settings for ADB paths, polling intervals, and log buffer limits.
- **🎨 Premium UI:** Frameless design with a custom titlebar, skeleton loaders, and smooth view transitions.

---

## 🛠 Tech Stack

- **Frontend:** [React 19](https://react.dev/), [Typescript](https://www.typescriptlang.org/), [Zustand](https://github.com/pmndrs/zustand) (State), [Lucide React](https://lucide.dev/) (Icons).
- **Backend:** [Rust](https://www.rust-lang.org/) (Core logic & ADB interaction).
- **Framework:** [Tauri v2](https://tauri.app/) (Lightweight desktop bridge).
- **Styling:** Vanilla CSS 3 with custom design tokens.

---

## 🚀 Getting Started

### Prerequisites

- **ADB (Android Debug Bridge):** Ensure ADB is installed and added to your system PATH.
- **Webview2 (Windows only):** Required for the frontend rendering.

### Installation

#### Linux (Debian/Ubuntu)
1. Download the `.deb` package from the [Releases](https://github.com/Watash1no/openquest_project/releases) page.
2. Install via terminal:
   ```bash
   sudo dpkg -i openquest-hub_*.deb
   ```

#### Windows
1. Download the `.exe` or `.msi` from the latest release.
2. Run the installer and follow the prompts.

---

## 👨‍💻 Development

If you want to build the project from source:

1. **Clone the repository:**
   ```bash
   git clone https://github.com/Watash1no/openquest_project.git
   cd openquest-hub
   ```

2. **Install dependencies:**
   ```bash
   npm install
   ```

3. **Run in development mode:**
   ```bash
   npm run tauri dev
   ```

4. **Build production bundle:**
   ```bash
   npm run tauri build
   ```

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

Contributions are welcome! Please open an issue or submit a pull request for any improvements.

---

*Developed with ❤️ for the VR Community.*
