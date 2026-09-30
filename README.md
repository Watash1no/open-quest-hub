# 🚀 OpenQuest Hub

**OpenQuest Hub** is a powerful, modern, and cross-platform desktop application designed for seamless management of Meta Quest (and other Android-based) VR headsets. Built with **Tauri**, **Rust**, and **React**, it provides a premium user experience for developers and enthusiasts alike.



<img width="953" height="1247" alt="image" src="https://github.com/user-attachments/assets/e10b7bcd-5f9d-4166-8c5f-223e602e516e" />


---

## ✨ Key Features

- **📱 Device Management:** Real-time polling of connected devices with status indicators (Online, Unauthorized, Offline).
- **📺 Screen Casting:** Real-time low-latency screen mirroring of your VR headset using built-in Scrcpy integration.
- **📦 App Explorer:** Browse, uninstall, and launch installed applications. Sideload APKs with a simple drag-and-drop.
- **📁 File Explorer:** Full access to device storage (`/sdcard`). Download files to your PC with real-time transfer progress and date-based sorting.
- **📟 Powerful Logcat:** High-performance log streaming with custom argument support, filtering, and the ability to export logs to a file.
- **🛡️ VR Settings:** Quick access to headset-specific settings like Boundary (Guardian) toggling and device-specific configurations.
- **⚙️ Deep Configuration:** Automated utility installation (Scrcpy, ADB), persistent settings for custom paths, and log buffer management.
- **🎨 Premium UI:** Modern, frameless design with custom animations, glassmorphism effects, and smooth view transitions.

### 🖥️ Terminal UI (TUI)

For headless setups, SSH sessions, and keyboard-driven workflows, a standalone Rust terminal app ships in [`openquest-tui/`](openquest-tui/README.md).

- **Pure terminal, no GUI required.** Built with Ratatui 0.28 and Crossterm 0.28; the same ADB commands as the desktop app, reimplemented for the terminal.
- **Six tabs, keyboard-first navigation.** Devices, Apps, Files, Install, Logcat, Settings — switch with `Tab` / `Shift+Tab`, press `?` for the help overlay.
- **Install APKs and push OBBs in one keypress.** The Install tab detects `.obb` files, parses the package name out of the filename, and pushes to `/sdcard/Android/obb/<pkg>/` automatically.
- **Mouse support included.** Click tabs, toggle file checkboxes, double-click to activate, scroll to navigate — all work alongside the hotkeys.

Build it from the `openquest-tui/` directory with `cargo build --release` and run `./target/release/openquest-tui`. See [openquest-tui/README.md](openquest-tui/README.md) for the full hotkey reference and Quest 3 quirks.

---

## 🛠 Tech Stack

- **Frontend:** [React 19](https://react.dev/), [Typescript](https://www.typescriptlang.org/), [Zustand](https://github.com/pmndrs/zustand) (State), [Lucide React](https://lucide.dev/) (Icons).
- **Backend:** [Rust](https://www.rust-lang.org/) (Core logic & ADB interaction).
- **Framework:** [Tauri v2](https://tauri.app/) (Lightweight desktop bridge).
- **Styling:** [Tailwind CSS 4](https://tailwindcss.com/) with modern design principles.

---

## 📱 Device Setup

To ensure **OpenQuest Hub** works correctly, your device must be properly configured.

### 🥽 Meta Quest Specific Setup
1.  **Create a Developer Organization:**
    - Go to [dashboard.oculus.com](https://dashboard.oculus.com/) and log in with your Meta account.
    - Create a "New Organization" (give it any name).
    - You may need to verify your account with a phone number or credit card.
2.  **Enable Developer Mode in the Mobile App:**
    - Open the **Meta Quest app** on your smartphone.
    - Go to **Menu > Devices** and select your headset.
    - Tap **Headset Settings > Developer Mode**.
    - Toggle the switch to **ON**.
3.  **Enable USB Debugging in the Headset:**
    - Connect the headset to your PC via USB.
    - Put on the headset and select **"Allow USB Debugging"** when the prompt appears (check "Always allow from this computer").

### 📱 Generic Android Device Setup
1.  **Enable Developer Options:** Go to **Settings > System > About** and tap **Build Number** 7 times.
2.  **Toggle Developer Mode:** Go to the newly appeared **Developer Options** menu and ensure **Developer Mode** and **USB Debugging** are turned **ON**.

---

## 🚀 Getting Started

### Prerequisites

- **Webview2 (Windows only):** Required for the frontend rendering.

### Installation

#### macOS
1. Download the `.dmg` from the [Releases](https://github.com/Watash1no/open-quest-hub/releases) page.
2. Open the `.dmg` and drag **Open Quest Hub** to your **Applications** folder.
3. See [First Launch](#-macos-1) below for instructions on how to bypass security warnings.

#### Windows
1. Download the `.exe` or `.msi` from the latest release.
2. Run the installer and follow the prompts.

#### Linux (Debian/Ubuntu)
1. Download the `.deb` package from the latest release.
2. Install via terminal:
   ```bash
   sudo dpkg -i openquest-hub_*.deb
   ```

#### 🖥️ Terminal UI (TUI)

A standalone Rust TUI binary ships in every release alongside the GUI. It uses the same ADB commands and is perfect for headless setups, SSH sessions, or anyone who prefers the terminal.

| Platform | Asset |
| --- | --- |
| Linux x86_64 | `openquest-tui-linux-x86_64` |
| macOS (Apple Silicon) | `openquest-tui-macos-aarch64` |
| Windows x86_64 | `openquest-tui-windows-x86_64.exe` |

Download the asset for your platform from the [Releases](https://github.com/Watash1no/open-quest-hub/releases) page, then:

```bash
# Linux / macOS
chmod +x openquest-tui-linux-x86_64   # or openquest-tui-macos-aarch64
./openquest-tui-linux-x86_64
```

```powershell
# Windows
.\openquest-tui-windows-x86_64.exe
```

For the full hotkey reference and Quest 3 quirks, see [openquest-tui/README.md](openquest-tui/README.md).

---

## 👨‍💻 Development

If you want to build the project from source:

1. **Clone the repository:**
   ```bash
   git clone https://github.com/Watash1no/open-quest-hub.git
   cd open-quest-hub
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

5. **Build the TUI binary** (optional, no Node/Tauri needed):
   ```bash
   cd openquest-tui
   cargo build --release
   ./target/release/openquest-tui
   ```

The TUI is a standalone Rust crate. See [openquest-tui/README.md](openquest-tui/README.md) for details.

---

## 📄 License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.

## 🤝 Contributing

Contributions are welcome! Please open an issue or submit a pull request for any improvements.

---

## 🚀 Installation & First Launch

Since the application is currently not signed with official developer certificates, you might see security warnings on your first launch. This is normal for early-stage open-source projects.

### 🍎 macOS
1. Download the `.dmg` and drag **Open Quest Hub** to your **Applications** folder.
2. **First Launch:** Since the app is not signed, macOS will block it with a "could not verify" message.
3. **How to bypass:**
   - **Option A (System Settings):**
     1. Try to open the app (it will fail).
     2. Open **System Settings** > **Privacy & Security**.
     3. Scroll down to the **Security** section.
     4. Click **Open Anyway** next to the "Open Quest Hub was blocked" message.
     5. Enter your password and click **Open** in the final dialog.
   - **Option B (Terminal):**
     1. Open **Terminal** and run:
        ```bash
        sudo xattr -rd com.apple.quarantine /Applications/Open\ Quest\ Hub.app
        ```
     2. Now the app will open with a normal double-click.

### 🪟 Windows
1. Download the `.exe` or `.msi` from the latest release.
2. Run the installer. If you see a blue "Windows protected your PC" screen:
   - Click **More info**.
   - Click **Run anyway**.

### 🐧 Linux
1. **Permissions:** To allow the app to access your VR headset via USB, you need to add a udev rule:
   ```bash
   echo 'SUBSYSTEM=="usb", ATTR{idVendor}=="2833", MODE="0666", GROUP="plugdev"' | sudo tee /etc/udev/rules.d/51-android.rules
   sudo udevadm control --reload-rules
   ```
2. **AppImage:**
   - Download the `.AppImage`.
   - Make it executable: `chmod +x openquest-hub.AppImage`.
   - Run it!
3. **Debian/Ubuntu:**
   - Download the `.deb` and install it: `sudo dpkg -i openquest-hub_*.deb`.

---

*Developed with ❤️ for the VR Community.*

---

<div align="center">
  <a href="https://buymeacoffee.com/watash1no"><img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-ffdd00?style=for-the-badge&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me A Coffee" /></a><a href="LICENSE"><img src="https://img.shields.io/github/license/Watash1no/open-quest-hub?style=for-the-badge" alt="License" /></a>
</div>




