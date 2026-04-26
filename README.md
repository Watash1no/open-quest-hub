# 🚀 OpenQuest Hub

**OpenQuest Hub** is a powerful, modern, and cross-platform desktop application designed for seamless management of Meta Quest (and other Android-based) VR headsets. Built with **Tauri**, **Rust**, and **React**, it provides a premium user experience for developers and enthusiasts alike.


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
1. Download the `.dmg` from the [Releases](https://github.com/Watash1no/openquest_project/releases) page.
2. Open the `.dmg` and drag **Open Quest Hub** to your **Applications** folder.

#### Windows
1. Download the `.exe` or `.msi` from the latest release.
2. Run the installer and follow the prompts.

#### Linux (Debian/Ubuntu)
1. Download the `.deb` package from the latest release.
2. Install via terminal:
   ```bash
   sudo dpkg -i openquest-hub_*.deb
   ```

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

## 🚀 Installation & First Launch

Since the application is currently not signed with official developer certificates, you might see security warnings on your first launch. This is normal for early-stage open-source projects.

### 🍎 macOS
1. Download the `.dmg` from the [Releases](https://github.com/Watash1no/openquest_project/releases) page.
2. Drag **Open Quest Hub** to your **Applications** folder.
3. **Important:** To launch for the first time, **Right-click** the app icon and select **Open**. When the warning dialog appears, click **Open** again.

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
  <a href="https://buymeacoffee.com/watash1no">
    <img src="https://img.shields.io/badge/Buy%20Me%20a%20Coffee-ffdd00?style=for-the-badge&logo=buy-me-a-coffee&logoColor=black" alt="Buy Me A Coffee" />
  </a>
  <img src="https://img.shields.io/github/license/watash1no/openquest_project?style=for-the-badge" alt="License" />
</div>

## License

This project is licensed under the MIT License - see the [LICENSE](LICENSE) file for details.


