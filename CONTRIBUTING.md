# Contributing to Open Quest Hub

Thanks for your interest in contributing to Open Quest Hub! We welcome all contributions, from bug reports to new features.

## Getting Started

### Prerequisites
- [Node.js](https://nodejs.org/) (Latest LTS)
- [Rust](https://www.rust-lang.org/tools/install) (latest stable version)
- macOS, Windows, or Linux development environment

### Setup
1. Clone the repository:
   ```bash
   git clone https://github.com/Watash1no/open-quest-hub.git
   cd open-quest-hub
   ```
2. Install dependencies:
   ```bash
   npm install
   ```

## Development
To run the application in development mode with hot-reloading:
```bash
npm run tauri dev
```

## Building
To build the production application:
```bash
npm run build:app
```

## Project Structure
- `src/`: Frontend React application (TypeScript, Tailwind CSS)
- `src-tauri/`: Backend Rust application (Tauri commands, ADB integration)
- `docs/`: Project documentation

## Submitting Changes
1. Fork the repository
2. Create a new branch for your feature or bug fix
3. Commit your changes with descriptive messages
4. Push your branch and open a Pull Request
5. Ensure your PR follows the [Pull Request Template](.github/PULL_REQUEST_TEMPLATE.md)

## Coding Standards
- Use TypeScript for frontend logic
- Use Rust for backend/system-level operations
- Maintain consistent styling using the existing Tailwind CSS configuration
