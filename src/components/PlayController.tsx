import { useState, useEffect } from 'react'
import { Play, Pause, Square, SkipForward, SkipBack, Volume2, VolumeX } from 'lucide-react'
import { invoke } from '@tauri-apps/api/tauri'

interface PlaybackState {
  is_playing: boolean
  current_audio_id: number | null
  current_audio_name: string | null
  volume: number
  speed: number
  playlist_queue: number[]
  current_index: number
  is_auto_play: boolean
}

export default function PlayController() {
  const [playbackState, setPlaybackState] = useState<PlaybackState | null>(null)
  const [isMuted, setIsMuted] = useState(false)

  const SPEED_OPTIONS = [0.8, 1.0, 1.2, 1.5, 2.0, 3.0]

  // 定期获取播放状态
  useEffect(() => {
    const updateState = async () => {
      try {
        const state = await invoke<PlaybackState>('get_playback_state')
        setPlaybackState(state)
      } catch (error) {
        console.error('获取播放状态失败:', error)
      }
    }

    updateState()
    const interval = setInterval(updateState, 1000) // 每秒更新一次

    return () => clearInterval(interval)
  }, [])

  const handlePlayPause = async () => {
    try {
      if (playbackState?.is_playing) {
        await invoke('pause_audio')
      } else {
        // 如果有当前音频，恢复播放需要重新调用 play_audio
        if (playbackState?.current_audio_id) {
          await invoke('play_audio', { id: playbackState.current_audio_id })
        }
      }
    } catch (error) {
      console.error('播放控制失败:', error)
    }
  }

  const handleStop = async () => {
    try {
      await invoke('stop_audio')
    } catch (error) {
      console.error('停止播放失败:', error)
    }
  }

  const handlePrevious = async () => {
    try {
      await invoke('play_previous')
    } catch (error) {
      console.error('上一曲失败:', error)
    }
  }

  const handleNext = async () => {
    try {
      await invoke('play_next')
    } catch (error) {
      console.error('下一曲失败:', error)
    }
  }

  const handleVolumeChange = async (newVolume: number) => {
    try {
      await invoke('set_volume', { volume: newVolume / 100 })
    } catch (error) {
      console.error('音量调节失败:', error)
    }
  }

  const toggleMute = async () => {
    const newMuted = !isMuted
    setIsMuted(newMuted)
    try {
      const currentVolume = playbackState?.volume || 0.5
      await invoke('set_volume', { volume: newMuted ? 0 : currentVolume })
    } catch (error) {
      console.error('静音切换失败:', error)
    }
  }

  const handleSpeedChange = async (speed: number) => {
    try {
      await invoke('set_speed', { speed })
    } catch (error) {
      console.error('倍速设置失败:', error)
    }
  }

  if (!playbackState || !playbackState.current_audio_id) {
    return null // 没有播放内容时不显示
  }

  const volume = Math.round(playbackState.volume * 100)
  const hasPlaylist = playbackState.playlist_queue.length > 0
  const canGoBack = hasPlaylist && playbackState.current_index > 0
  const canGoForward = hasPlaylist && playbackState.current_index < playbackState.playlist_queue.length - 1

  return (
    <div className="bg-white border-t border-gray-200 p-4">
      <div className="max-w-7xl mx-auto">
        {/* 当前播放信息 */}
        <div className="mb-3">
          <div className="flex items-center gap-3 text-sm mb-1">
            <span className="font-medium text-gray-800">
              {playbackState.current_audio_name || '未知音频'}
            </span>
            {playbackState.is_auto_play && (
              <span className="px-2 py-0.5 bg-blue-100 text-blue-700 rounded text-xs">
                定时播放中
              </span>
            )}
            {hasPlaylist && (
              <span className="text-gray-500 text-xs">
                {playbackState.current_index + 1} / {playbackState.playlist_queue.length}
              </span>
            )}
            <div className="flex-1" />
            <span className="text-gray-600">
              倍速: {playbackState.speed.toFixed(1)}x
            </span>
          </div>
        </div>

        {/* 控制按钮 */}
        <div className="flex items-center justify-between">
          <div className="flex items-center gap-2">
            {/* 停止按钮 */}
            <button
              onClick={handleStop}
              className="p-2 hover:bg-gray-100 rounded-lg transition-colors"
              title="停止"
            >
              <Square size={20} className="text-gray-700" />
            </button>

            {/* 上一曲 */}
            <button
              onClick={handlePrevious}
              disabled={!canGoBack}
              className={`p-2 rounded-lg transition-colors ${
                canGoBack
                  ? 'hover:bg-gray-100 text-gray-700'
                  : 'opacity-30 cursor-not-allowed text-gray-400'
              }`}
              title="上一曲"
            >
              <SkipBack size={20} />
            </button>

            {/* 播放/暂停 */}
            <button
              onClick={handlePlayPause}
              className="p-3 bg-blue-600 text-white hover:bg-blue-700 rounded-lg transition-colors"
              title={playbackState.is_playing ? '暂停' : '播放'}
            >
              {playbackState.is_playing ? <Pause size={24} /> : <Play size={24} />}
            </button>

            {/* 下一曲 */}
            <button
              onClick={handleNext}
              disabled={!canGoForward}
              className={`p-2 rounded-lg transition-colors ${
                canGoForward
                  ? 'hover:bg-gray-100 text-gray-700'
                  : 'opacity-30 cursor-not-allowed text-gray-400'
              }`}
              title="下一曲"
            >
              <SkipForward size={20} />
            </button>
          </div>

          <div className="flex items-center gap-4">
            {/* 倍速控制 */}
            <div className="flex items-center gap-1">
              {SPEED_OPTIONS.map((speed) => (
                <button
                  key={speed}
                  onClick={() => handleSpeedChange(speed)}
                  className={`px-2 py-1 text-xs rounded transition-colors ${
                    Math.abs(playbackState.speed - speed) < 0.01
                      ? 'bg-blue-600 text-white'
                      : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
                  }`}
                  title={`${speed}倍速`}
                >
                  {speed}x
                </button>
              ))}
            </div>

            {/* 音量控制 */}
            <div className="flex items-center gap-2">
              <button
                onClick={toggleMute}
                className="p-2 hover:bg-gray-100 rounded-lg transition-colors"
                title={isMuted ? '取消静音' : '静音'}
              >
                {isMuted ? <VolumeX size={20} /> : <Volume2 size={20} />}
              </button>

              <input
                type="range"
                min="0"
                max="100"
                value={volume}
                onChange={(e) => handleVolumeChange(parseInt(e.target.value))}
                className="w-24"
              />

              <span className="text-sm text-gray-600 w-10">{volume}%</span>
            </div>
          </div>
        </div>
      </div>
    </div>
  )
}
