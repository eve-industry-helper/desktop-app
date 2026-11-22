# desktop-app
Repository of the main application.

## Prerequisites

Before developing or building the application, ensure you have the following installed on your system:

* **[Node.js](https://nodejs.org/)** (LTS recommended) & **[pnpm](https://pnpm.io/installation)**
* **[Rust & Cargo](https://www.rust-lang.org/tools/install)** (via `rustup`)
* **Tauri System Dependencies:**
  * **Windows:** [Microsoft C++ Build Tools](https://visualstudio.microsoft.com/visual-cpp-build-tools/) and [WebView2](https://developer.microsoft.com/en-us/microsoft-edge/webview2/) (usually pre-installed on Windows 10/11).
  * **macOS:** Xcode Command Line Tools (`xcode-select --install`).
  * **Linux:** Standard build tools and WebKitGTK libraries (see the [Tauri Linux prerequisites](https://v2.tauri.app/start/prerequisites/#linux) for your distro, e.g., `webkit2gtk-4.1`, `base-devel` / `build-essential`, `libappindicator`, `librsvg`).

---

## Development Setup (For Developers)

1. **Install frontend dependencies:**
   From the project root, run:
   ```bash
   pnpm install
   ```

2. **Start the development server:**
   Launch the SvelteKit frontend and Tauri Rust backend together with hot-reload enabled:
   ```bash
   pnpm tauri dev
   ```
   *Note: The first launch will take a few minutes while Cargo downloads and compiles the Rust crates. Subsequent runs will be much faster.*

3. **Type-checking (optional):**
   To verify TypeScript and Svelte types without launching the full desktop window:
   ```bash
   pnpm check
   ```

---

## Building from Source (For Users)

1. **Install dependencies:**
   From the project root, run:
   ```bash
   pnpm install
   ```

2. **Compile the application:**
   Run the production build command:
   ```bash
   pnpm tauri build
   ```
   This compiles the SvelteKit frontend into static assets, builds the optimised Rust binary, and packages native installers for your operating system.

3. **Locate your installer or executable:**
   Once the build finishes, the compiled files are located in `src-tauri/target/release/bundle/`:
   * **Windows:** `.msi` and `.exe` (NSIS) installers in `msi/` and `nsis/`
   * **macOS:** `.dmg` and `.app` bundles in `dmg/` and `macos/`
   * **Linux:** `.deb`, `.rpm`, and `.AppImage` packages in `deb/`, `rpm/`, and `appimage/`

   If you only want the standalone binary without an installer, it can be found directly at `src-tauri/target/release/`.
