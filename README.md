# iMAGE

A Google Photos-like mobile application for browsing files on EC2 instances. Connect via SSH using your PEM key and browse photos, videos, documents, and folders in a beautiful, intuitive interface.

## Features

- **Mobile-first design** - Optimized for iOS and Android
- **SSH Connection** - Securely connect to EC2 instances using PEM keys
- **Google Photos-like UI** - Beautiful dark theme with album and grid views
- **Photo/Video Viewer** - Built-in viewer for images and videos
- **Album Navigation** - Browse folders as albums with breadcrumb navigation
- **Media Filter** - Toggle to show only photos and videos
- **Offline Support** - Saves connection details locally

## Tech Stack

- **Frontend**: Vue.js 3 + TypeScript + Vite
- **Backend**: Rust + Tauri 2.0
- **SSH**: ssh2 crate for secure connections
- **Mobile**: Tauri Mobile (iOS/Android)
- **State Management**: Pinia
- **Icons**: Lucide Vue
- **Package Manager**: pnpm

## Getting Started

### Prerequisites

- Node.js 20+
- pnpm 9+
- Rust 1.75+
- Xcode (for iOS development - macOS only)
- Android Studio (for Android development)

### Installation

```bash
# Navigate to project
cd iMAGE

# Install dependencies
pnpm install

# Run development server
pnpm tauri:dev

# For mobile development
pnpm tauri:android:dev
pnpm tauri:ios:dev
```

### Building

```bash
# Build for desktop
pnpm tauri:build

# Build for Android
pnpm tauri:build:android

# Build for iOS (macOS only)
pnpm tauri:build:ios
```

## Usage

1. **Connect**: Enter your EC2 instance details:
   - IP Address or hostname
   - Username (ec2-user, ubuntu, root, etc.)
   - SSH port (default: 22)
   - PEM key file

2. **Browse**: Navigate through folders (shown as albums)

3. **View**: Click on photos or videos to view them

4. **Filter**: Toggle "Photos/Videos Only" mode to hide other files

## GitHub Actions

This repository includes GitHub Actions workflows for building unsigned mobile packages:

- **Build iOS** (`.github/workflows/build-ios.yml`): Builds unsigned IPA on macOS runners
- **Build Android** (`.github/workflows/build-android.yml`): Builds APK on Ubuntu runners

Both workflows run on:
- Push to `main` or `develop` branches
- Pull requests to `main`
- Manual trigger via `workflow_dispatch`

### Build Outputs

- **IPA**: Available as `iMAGE-unsigned-ipa` artifact
- **APK**: Available as `iMAGE-apks` artifact (includes debug and release variants)

## Project Structure

```
iMAGE/
├── .github/workflows/    # CI/CD workflows
├── src/                  # Vue.js frontend
│   ├── components/       # Vue components
│   ├── views/           # Page views
│   ├── stores/          # Pinia stores
│   ├── router/          # Vue Router
│   └── style.css        # Global styles
├── src-tauri/           # Rust backend
│   └── src/
│       ├── main.rs      # Entry point
│       ├── ssh.rs       # SSH connection logic
│       └── commands.rs  # Tauri commands
└── package.json
```

## Security Notes

- PEM keys are stored locally on your device
- All SSH connections are encrypted
- No data is sent to external servers
- Connection details are optionally saved locally

## License

MIT

## Support

For issues and feature requests, please use the GitHub issue tracker.
