import { createContext, useContext, useState, useEffect, ReactNode } from 'react'
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

interface PlayerContextType {
  isPlaying: boolean
  currentAudio: {
    id: number
    name: string
  } | null
  playAudio: (id: number, name: string) => Promise<void>
  pauseAudio: () => Promise<void>
  stopAudio: () => Promise<void>
  togglePlayPause: () => Promise<void>
}

const PlayerContext = createContext<PlayerContextType | undefined>(undefined)

export function PlayerProvider({ children }: { children: ReactNode }) {
  const [isPlaying, setIsPlaying] = useState(false)
  const [currentAudio, setCurrentAudio] = useState<{ id: number; name: string } | null>(null)

  // 定期同步播放状态
  useEffect(() => {
    const syncState = async () => {
      try {
        const state = await invoke<PlaybackState>('get_playback_state')

        // 只在状态真正改变时更新，避免不必要的重渲染
        if (state.is_playing !== isPlaying) {
          setIsPlaying(state.is_playing)
        }

        // 如果后端没有当前音频（播放完成或停止），清除前端状态
        if (!state.current_audio_id && currentAudio) {
          setCurrentAudio(null)
          setIsPlaying(false)
        } else if (state.current_audio_id && state.current_audio_name) {
          // 如果后端有音频但前端没有，或者ID不匹配，更新前端状态
          if (!currentAudio || currentAudio.id !== state.current_audio_id) {
            setCurrentAudio({
              id: state.current_audio_id,
              name: state.current_audio_name
            })
          }
        }
      } catch (error) {
        console.error('同步播放状态失败:', error)
      }
    }

    syncState()
    const interval = setInterval(syncState, 500) // 每0.5秒同步一次

    return () => clearInterval(interval)
  }, [isPlaying, currentAudio])

  const playAudio = async (id: number, name: string) => {
    try {
      await invoke('play_audio', { id })
      setCurrentAudio({ id, name })
      setIsPlaying(true)
    } catch (error) {
      console.error('播放失败:', error)
      throw error
    }
  }

  const pauseAudio = async () => {
    try {
      await invoke('pause_audio')
      setIsPlaying(false)
    } catch (error) {
      console.error('暂停失败:', error)
      throw error
    }
  }

  const stopAudio = async () => {
    try {
      await invoke('stop_audio')
      setIsPlaying(false)
      setCurrentAudio(null)
    } catch (error) {
      console.error('停止失败:', error)
      throw error
    }
  }

  const togglePlayPause = async () => {
    if (isPlaying) {
      await pauseAudio()
    } else {
      if (currentAudio) {
        // 如果有当前音频，恢复播放
        await invoke('play_audio', { id: currentAudio.id })
        setIsPlaying(true)
      }
    }
  }

  return (
    <PlayerContext.Provider
      value={{
        isPlaying,
        currentAudio,
        playAudio,
        pauseAudio,
        stopAudio,
        togglePlayPause,
      }}
    >
      {children}
    </PlayerContext.Provider>
  )
}

export function usePlayer() {
  const context = useContext(PlayerContext)
  if (context === undefined) {
    throw new Error('usePlayer must be used within a PlayerProvider')
  }
  return context
}
