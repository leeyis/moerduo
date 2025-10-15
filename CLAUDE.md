# CLAUDE.md

This file provides guidance to Claude Code (claude.ai/code) when working with code in this repository.

## Project Overview

Moerduo (磨耳朵) is a cross-platform desktop application for timed English listening practice, built with Tauri, React, and Rust. The app allows students to schedule audio playback for language learning.

## Tech Stack

**Frontend:** React 18 + TypeScript + Tailwind CSS + Zustand + Vite
**Backend:** Tauri 1.5 + Rust + SQLite (rusqlite) + Rodio (audio) + Tokio (async)

## Development Commands

### Running the Application
```bash
cd moerduo
npm run dev              # Run Tauri dev mode (recommended)
npm run dev:web          # Run Vite dev server only (frontend preview)
```

### Building
```bash
npm run build            # Build production Tauri app
npm run build:web        # Build frontend only
```

### Installation
```bash
npm install              # Install frontend dependencies
# Cargo dependencies auto-download on first build
```

## Architecture

### Frontend Structure
- **src/pages/**: Page components (AudioLibrary, Playlists, Tasks, Statistics, Settings, Help)
- **src/components/**: Reusable components (PlayController, DeleteConfirmDialog)
- **src/contexts/**: React contexts (PlayerContext for global audio state)
- **src/hooks/**: Custom hooks (useTheme)

### Backend Structure (src-tauri/src/)
- **main.rs**: Entry point, manages shared state (Arc<Mutex<Connection>>, AudioPlayer), starts scheduler
- **db.rs**: SQLite database initialization and schema
- **audio.rs**: Audio file management (upload, delete, scan)
- **player.rs**: Rodio-based audio playback engine
- **playlist.rs**: Playlist CRUD operations
- **task.rs**: Scheduled task management
- **scheduler.rs**: Background task scheduler (tokio-based, runs at app startup)
- **stats.rs**: Usage statistics
- **settings.rs**: App settings and config import/export

### Database Schema
- **audio_files**: Audio file metadata with play counts
- **playlists**: Multiple playlists with play modes (sequential/random/single/loop)
- **playlist_items**: Many-to-many relationship between playlists and audio files
- **scheduled_tasks**: Timed playback tasks with repeat patterns (daily/weekday/weekend/custom/once)
- **execution_history**: Task execution logs for statistics
- **app_settings**: Key-value settings storage

### State Management
- **Frontend**: PlayerContext provides global audio player state across React components
- **Backend**: Shared state via Arc<Mutex<T>> for database connection and audio player
- **Communication**: Tauri commands bridge frontend and backend (invoked via @tauri-apps/api)

### Key Design Patterns
- **Separation of concerns**: Audio management, playback, scheduling, and UI are independent modules
- **Async runtime**: Tokio powers the scheduler which runs in the background checking for tasks
- **Thread-safe sharing**: Arc<Mutex<T>> allows safe concurrent access to database and player

## Important Implementation Details

### Audio File Storage
- Files are stored in `{app_data_dir}/audio/` with UUIDs as filenames
- Original filenames and metadata stored in SQLite
- Supported formats: MP3, WAV, OGG, FLAC, M4A

### Task Scheduling
- Scheduler runs in a tokio task spawned at app startup (main.rs:41)
- Checks enabled tasks periodically and triggers playback at scheduled times
- Supports fade-in (gradual volume increase over N seconds)
- Custom repeat patterns stored as JSON in `custom_days` field

### Tauri Commands
All backend functions are exposed via `#[tauri::command]` and registered in main.rs:52-81. Frontend invokes them using:
```typescript
import { invoke } from '@tauri-apps/api/tauri'
await invoke('get_audio_files', { /* params */ })
```

## Current Development Status

**Completed (60% overall):**
- Full audio file management
- Playlist CRUD with play modes
- Task management UI
- Basic audio playback
- Database schema and initialization

**In Progress:**
- Task scheduler auto-trigger logic
- Alarm notification window
- System tray integration

**Pending:**
- Statistics implementation
- Settings UI (auto-start, theme, backup/restore)
- Audio duration detection (currently uses default values)
- Playback speed control
- System wake-up on Windows (requires Windows Task Scheduler API)

## Testing Approach

When testing:
1. Upload various audio formats to test audio management
2. Create playlists with different play modes
3. Create tasks with different repeat patterns and verify they're stored correctly
4. Test manual playback before testing scheduled playback
5. Check database state using SQLite tools if needed (db located in app data dir)

## Common Patterns

### Adding a New Tauri Command
1. Define function with `#[tauri::command]` in appropriate module (e.g., audio.rs)
2. Add to `invoke_handler` in main.rs
3. Call from frontend using `invoke('command_name', { params })`

### Accessing Shared State in Tauri Commands
```rust
#[tauri::command]
async fn my_command(
    db: tauri::State<'_, Arc<Mutex<Connection>>>,
    player: tauri::State<'_, Arc<Mutex<AudioPlayer>>>,
) -> Result<(), String> {
    let conn = db.lock().await;
    let player = player.lock().await;
    // use conn and player
    Ok(())
}
```

### Database Queries
Always use parameterized queries to prevent SQL injection:
```rust
conn.execute(
    "INSERT INTO table (col) VALUES (?1)",
    params![value],
)?;
```

## Platform-Specific Considerations

- **Windows**: System wake feature uses Win32 API (windows crate dependency)
- **Paths**: Use Tauri's path resolver for cross-platform app data directories
- **Audio**: Rodio handles cross-platform audio, but format support may vary by OS
