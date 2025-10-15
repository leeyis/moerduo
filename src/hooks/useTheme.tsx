import { useState, useEffect } from 'react'
import { invoke } from '@tauri-apps/api/tauri'

export interface ThemeSettings {
  auto_start: boolean
  minimize_to_tray: boolean
  default_volume: number
  theme: string
  audio_path: string | null
}

export function useTheme() {
  const [settings, setSettings] = useState<ThemeSettings>({
    auto_start: false,
    minimize_to_tray: true,
    default_volume: 50,
    theme: 'light',
    audio_path: null,
  })

  const [isLoading, setIsLoading] = useState(true)

  useEffect(() => {
    loadSettings()
  }, [])

  const loadSettings = async () => {
    try {
      const settingsData = await invoke<ThemeSettings>('get_settings')
      setSettings(settingsData)

      // 应用主题
      applyTheme(settingsData.theme)
    } catch (error) {
      console.error('加载设置失败:', error)
      // 使用默认设置
      applyTheme('light')
    } finally {
      setIsLoading(false)
    }
  }

  const applyTheme = (theme: string) => {
    if (theme === 'auto') {
      // 跟随系统主题
      const prefersDark = window.matchMedia('(prefers-color-scheme: dark)').matches
      document.documentElement.setAttribute('data-theme', prefersDark ? 'dark' : 'light')

      // 监听系统主题变化
      const mediaQuery = window.matchMedia('(prefers-color-scheme: dark)')
      const handleChange = (e: MediaQueryListEvent) => {
        document.documentElement.setAttribute('data-theme', e.matches ? 'dark' : 'light')
      }
      mediaQuery.addEventListener('change', handleChange)

      // 清理函数会在组件卸载时调用
      return () => {
        mediaQuery.removeEventListener('change', handleChange)
      }
    } else {
      // 手动设置主题
      document.documentElement.setAttribute('data-theme', theme)
    }
  }

  const saveSetting = async (key: string, value: any) => {
    try {
      await invoke('save_setting', { key, value: value.toString() })
      setSettings(prev => ({ ...prev, [key]: value }))

      // 如果是主题设置，立即应用
      if (key === 'theme') {
        applyTheme(value)
      }
    } catch (error) {
      console.error('保存设置失败:', error)
    }
  }

  const saveSettings = async (newSettings: Partial<ThemeSettings>) => {
    try {
      await invoke('save_settings', { settings: newSettings })
      setSettings(prev => ({ ...prev, ...newSettings }))

      // 应用主题
      if (newSettings.theme) {
        applyTheme(newSettings.theme)
      }
    } catch (error) {
      console.error('保存设置失败:', error)
    }
  }

  // 初始应用主题和系统监听
  useEffect(() => {
    let cleanup: (() => void) | undefined

    if (settings.theme === 'auto') {
      cleanup = applyTheme(settings.theme)
    } else {
      applyTheme(settings.theme)
    }

    return cleanup
  }, [settings.theme])

  return {
    settings,
    isLoading,
    setSettings,
    saveSetting,
    saveSettings,
    loadSettings,
  }
}