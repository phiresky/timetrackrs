# Copilot Instructions for timetrackrs

## Repository Overview

timetrackrs is an automatic rule-based time tracker that records what you spend time on and stores it in a database. It provides a Web UI to analyze activity and create custom classification rules.

**Key concepts:**
- User activity tracked as "events" with timestamps, duration, and tags
- Tags are key-value pairs (e.g., `category:Productivity/Software Development/IDE`)
- Rules add derived tags based on existing tags
- External fetchers retrieve additional metadata (e.g., YouTube video categories)

## Technology Stack

**Backend (Rust):**
- Rust 2021 edition
- SQLite with sqlx for database operations
- Tokio for async runtime
- Warp for HTTP server
- Multiple data sources for tracking (X11, Windows, browser usage, etc.)

**Frontend (TypeScript/React):**
- React 18 with TypeScript
- MobX for state management
- Vite for build tooling
- Plotly.js for data visualization
- Bootstrap 4 and Reactstrap for UI components
- Yarn v4 for package management

## Build Instructions

### Prerequisites
- Rust toolchain (cargo, rustc)
- Node.js and Yarn v4
- SQLite3

### Initial Setup
**ALWAYS run these steps in order for a fresh clone:**

1. **Generate development database schemas (REQUIRED before building):**
   ```bash
   ./update-schemas.sh
   ```
   This script creates SQLite databases needed for sqlx compile-time verification. It must be run before any Rust compilation.

2. **Build the frontend:**
   ```bash
   cd frontend
   yarn install
   yarn build
   cd ..
   ```

3. **Build the Rust backend:**
   ```bash
   cargo build
   ```

### Development Workflow

**Frontend development:**
```bash
cd frontend
yarn dev  # Starts Vite dev server with hot reload
```

**Backend development:**
```bash
# Run server without capturing, using debug config
RUST_LOG=debug,sqlx=warn,hyper=warn cargo run --bin timetrackrs -- --config data/debug_config.json
```

**Windows-specific note:** If building fails with openssl-sys errors, use:
```bash
cargo install --features openssl-vendored --path .
```

### Linting and Code Quality

**Frontend:**
```bash
cd frontend
yarn lint  # Runs ESLint and ts-prune
```

**Backend:**
- Rust code follows standard Rust formatting
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting

### Testing

Currently, there is no formal test infrastructure in the repository. Changes should be validated by:
1. Building successfully (both frontend and backend)
2. Running the application and verifying functionality manually
3. Checking that existing functionality is not broken

## Project Layout

### Root Directory Structure
```
/
├── .github/           # GitHub-specific files (workflows, instructions)
├── src/               # Rust backend source code
│   ├── bin/          # Binary entry points
│   ├── capture/      # Data capture modules (X11, Windows, etc.)
│   ├── db/           # Database operations
│   ├── events/       # Event handling
│   ├── extract/      # Data extraction logic
│   ├── import/       # Import functionality
│   └── server/       # Web server and API
├── frontend/         # TypeScript/React frontend
│   └── src/         # Frontend source code
├── migrations/       # SQL migration files
├── data/            # Data and configuration files
├── docs/            # Documentation and screenshots
├── Cargo.toml       # Rust project configuration
├── build.rs         # Rust build script
└── update-schemas.sh # Database schema generation script
```

### Key Configuration Files
- `Cargo.toml` - Rust dependencies and project metadata
- `frontend/package.json` - Frontend dependencies and scripts
- `frontend/vite.config.ts` - Vite configuration
- `frontend/tsconfig.json` - TypeScript configuration
- `frontend/.eslintrc.js` - ESLint configuration
- `.gitignore` - Git ignore patterns

### Important Files
- `README.md` - Comprehensive project documentation
- `build.rs` - Rust build script (handles embedded resources)
- `update-schemas.sh` - **CRITICAL**: Must be run before building to generate SQLx development databases
- `timetrackrs.service` - Systemd service file template

## Architecture

### Backend Architecture
- **Data Sources** (`src/capture/`): Capture raw activity data (window tracking, browser usage, etc.)
- **Events** (`src/events/`): Core event data structures and processing
- **Database** (`src/db/`): SQLite database operations using sqlx
- **Rules Engine** (`src/extract/`): Applies classification rules to add derived tags
- **External Fetchers**: Query external APIs (YouTube, Wikidata) for metadata
- **Web Server** (`src/server/`): Warp-based HTTP server serving API and frontend

### Frontend Architecture
- **MobX stores**: State management for application data
- **React components**: UI components using Reactstrap
- **Plotly visualizations**: Time-series plots and dashboards
- **API client**: Communicates with Rust backend

## Common Pitfalls and Workarounds

1. **Must run `./update-schemas.sh` before building**: The Rust backend uses sqlx with compile-time query verification. This requires development SQLite databases to exist before compilation.

2. **Frontend must be built before backend for production**: The Rust backend embeds the frontend assets using rust-embed. Build frontend first with `yarn build`.

3. **Multiple SQLite databases**: The project uses separate SQLite databases (raw_events, extracted, config) attached to a single connection. This is why standard sqlx migration CLI doesn't work.

4. **Platform-specific dependencies**: 
   - Linux: X11 and Wayland support
   - Windows: WinAPI dependencies
   - macOS: Core Foundation and Core Graphics

## Validation Steps

Before submitting changes:

1. **Run schema update if migrations changed:**
   ```bash
   ./update-schemas.sh
   ```

2. **Build frontend:**
   ```bash
   cd frontend && yarn install && yarn build && cd ..
   ```

3. **Build backend:**
   ```bash
   cargo build
   ```

4. **Lint frontend:**
   ```bash
   cd frontend && yarn lint
   ```

5. **Format Rust code:**
   ```bash
   cargo fmt
   ```

6. **Run clippy:**
   ```bash
   cargo clippy
   ```

## Additional Notes

- The project emphasizes storing raw data during capture and interpreting it during analysis
- Rule derivation chains can be viewed in the UI to understand how tags are derived
- External fetchers (YouTube, Wikidata) are used to enrich data with metadata
- The project uses Zstandard compression for efficient storage of event data

## Trust These Instructions

These instructions are comprehensive and up-to-date. Only search the codebase if:
- These instructions are incomplete for your specific task
- You encounter errors that contradict these instructions
- You need to understand implementation details not covered here
