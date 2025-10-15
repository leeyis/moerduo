use rusqlite::{Connection, Result};
use std::path::Path;

pub fn init_database(db_path: &Path) -> Result<Connection> {
    let conn = Connection::open(db_path)?;

    // 创建音频文件表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS audio_files (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            filename TEXT NOT NULL,
            original_name TEXT NOT NULL,
            file_path TEXT NOT NULL,
            file_size INTEGER NOT NULL,
            duration INTEGER NOT NULL,
            format TEXT NOT NULL,
            upload_date DATETIME DEFAULT CURRENT_TIMESTAMP,
            play_count INTEGER DEFAULT 0,
            last_played DATETIME
        )",
        [],
    )?;

    // 创建播放列表表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS playlists (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            play_mode TEXT DEFAULT 'sequential',
            created_date DATETIME DEFAULT CURRENT_TIMESTAMP,
            updated_date DATETIME DEFAULT CURRENT_TIMESTAMP
        )",
        [],
    )?;

    // 创建播放列表项表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS playlist_items (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            playlist_id INTEGER NOT NULL,
            audio_id INTEGER NOT NULL,
            sort_order INTEGER NOT NULL,
            FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE,
            FOREIGN KEY (audio_id) REFERENCES audio_files(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // 创建定时任务表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS scheduled_tasks (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            name TEXT NOT NULL,
            hour INTEGER NOT NULL,
            minute INTEGER NOT NULL,
            repeat_mode TEXT NOT NULL,
            custom_days TEXT,
            playlist_id INTEGER NOT NULL,
            volume INTEGER DEFAULT 50,
            fade_in_duration INTEGER DEFAULT 0,
            is_enabled BOOLEAN DEFAULT 1,
            priority INTEGER DEFAULT 0,
            created_date DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // 创建执行历史表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS execution_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            task_id INTEGER NOT NULL,
            execution_time DATETIME DEFAULT CURRENT_TIMESTAMP,
            status TEXT NOT NULL,
            duration INTEGER,
            FOREIGN KEY (task_id) REFERENCES scheduled_tasks(id) ON DELETE CASCADE
        )",
        [],
    )?;

    // 创建应用设置表
    conn.execute(
        "CREATE TABLE IF NOT EXISTS app_settings (
            key TEXT PRIMARY KEY,
            value TEXT NOT NULL
        )",
        [],
    )?;

    // 数据库迁移：为 scheduled_tasks 添加 duration_minutes 字段
    // 检查字段是否存在，如果不存在则添加
    let column_exists: Result<i64, _> = conn.query_row(
        "SELECT COUNT(*) FROM pragma_table_info('scheduled_tasks') WHERE name='duration_minutes'",
        [],
        |row| row.get(0),
    );

    if let Ok(count) = column_exists {
        if count == 0 {
            conn.execute(
                "ALTER TABLE scheduled_tasks ADD COLUMN duration_minutes INTEGER",
                [],
            )?;
        }
    }

    // 创建播放历史记录表（用于统计和日历展示）
    conn.execute(
        "CREATE TABLE IF NOT EXISTS playback_history (
            id INTEGER PRIMARY KEY AUTOINCREMENT,
            audio_id INTEGER NOT NULL,
            audio_name TEXT NOT NULL,
            playlist_id INTEGER,
            playlist_name TEXT,
            play_time DATETIME DEFAULT CURRENT_TIMESTAMP,
            FOREIGN KEY (audio_id) REFERENCES audio_files(id) ON DELETE CASCADE,
            FOREIGN KEY (playlist_id) REFERENCES playlists(id) ON DELETE SET NULL
        )",
        [],
    )?;

    Ok(conn)
}
