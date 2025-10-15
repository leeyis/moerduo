import { createContext, useContext, useState, ReactNode } from 'react'
import { invoke } from '@tauri-apps/api/tauri'

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
