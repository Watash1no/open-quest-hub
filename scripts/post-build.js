import fs from 'fs';
import path from 'path';
import { fileURLToPath } from 'url';
import { execSync } from 'child_process';

const __dirname = path.dirname(fileURLToPath(import.meta.url));
const root = path.resolve(__dirname, '..');

const platform = process.platform;
const productName = 'openquest-hub';
let binaryName = productName;

if (platform === 'win32') {
  binaryName += '.exe';
}

const destDir = path.join(root, 'builds');
console.log(`🚀 Running post-build for platform: ${platform}`);

// Ensure builds directory exists
if (!fs.existsSync(destDir)) {
  fs.mkdirSync(destDir, { recursive: true });
}

async function run() {
  // 1. Copy Primary Target (.app for Mac, .exe for Windows/Linux)
  let primarySrc;
  let primaryDest;
  let isDirectory = false;

  if (platform === 'darwin') {
    const appBundle = path.join(root, 'src-tauri', 'target', 'release', 'bundle', 'macos', `${productName}.app`);
    if (fs.existsSync(appBundle)) {
      primarySrc = appBundle;
      primaryDest = path.join(destDir, `${productName}.app`);
      isDirectory = true;
    }
  }

  if (!primarySrc) {
    primarySrc = path.join(root, 'src-tauri', 'target', 'release', binaryName);
    primaryDest = path.join(destDir, binaryName);
  }

  if (fs.existsSync(primarySrc)) {
    try {
      if (fs.existsSync(primaryDest)) {
        fs.rmSync(primaryDest, { recursive: true, force: true });
      }
      if (isDirectory) {
        fs.cpSync(primarySrc, primaryDest, { recursive: true });
        console.log(`✅ Successfully copied app bundle to: ${primaryDest}`);
      } else {
        fs.copyFileSync(primarySrc, primaryDest);
        console.log(`✅ Successfully copied binary to: ${primaryDest}`);
      }
    } catch (err) {
      console.error(`❌ Error copying primary target: ${err.message}`);
    }
  }

  // 2. Copy Installers (DMG for Mac)
  if (platform === 'darwin') {
    const dmgDir = path.join(root, 'src-tauri', 'target', 'release', 'bundle', 'dmg');
    if (fs.existsSync(dmgDir)) {
      const files = fs.readdirSync(dmgDir);
      const dmgFile = files.find(f => f.endsWith('.dmg'));
      if (dmgFile) {
        const dmgSrc = path.join(dmgDir, dmgFile);
        const dmgDest = path.join(destDir, dmgFile);
        
        // Clean old DMGs from builds dir
        const existingBuilds = fs.readdirSync(destDir);
        existingBuilds.forEach(f => {
          if (f.endsWith('.dmg')) {
            fs.unlinkSync(path.join(destDir, f));
          }
        });

        try {
          fs.copyFileSync(dmgSrc, dmgDest);
          console.log(`✅ Successfully copied DMG to: ${dmgDest}`);
          
          // 3. Try to close the Finder window and unmount the volume
          // This is a workaround for the DMG bundler leaving windows open
          console.log(`🧹 Attempting to close DMG windows and unmount...`);
          try {
            // Close Finder window if it's open
            execSync(`osascript -e 'tell application "Finder" to close (every window whose name is "${productName}")' 2>/dev/null || true`);
            // Detach volume if it's still mounted
            execSync(`hdiutil detach "/Volumes/${productName}" -force 2>/dev/null || true`);
          } catch (e) {
            // Ignore errors
          }

        } catch (err) {
          console.error(`❌ Error copying DMG: ${err.message}`);
        }
      }
    }
  }
}

run();
