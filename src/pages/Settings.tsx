import { useState } from 'react'
import { Save, FolderOpen, Moon, Sun, Volume2, Download, Upload } from 'lucide-react'
import { invoke } from '@tauri-apps/api/tauri'
import { open } from '@tauri-apps/api/dialog'
import { useTheme } from '../hooks/useTheme'

export default function SettingsPage() {
  const { settings, setSettings, saveSettings } = useTheme()
  const [saved, setSaved] = useState(false)

  const handleSave = async () => {
    try {
      // 保存普通设置
      await invoke('save_settings', { settings })

      // 单独处理开机自启动
      await invoke('set_auto_launch', { enable: settings.auto_start })

      setSaved(true)
      setTimeout(() => setSaved(false), 2000)
    } catch (error) {
      console.error('保存设置失败:', error)
      alert('保存设置失败: ' + error)
    }
  }

  const handleExportConfig = async () => {
    try {
      const filePath = await invoke<string>('export_config')
      alert(`配置已导出到: ${filePath}`)
    } catch (error) {
      console.error('导出失败:', error)
      alert('导出失败: ' + error)
    }
  }

  const handleImportConfig = async () => {
    try {
      const result = await invoke<string>('import_config')
      alert(result)
      // Reload page to get updated settings
      window.location.reload()
    } catch (error) {
      console.error('导入失败:', error)
      alert('导入失败: ' + error)
    }
  }

  const handleChangeAudioPath = async () => {
    try {
      const selected = await open({
        directory: true,
        multiple: false,
        title: '选择音频存储路径'
      })

      if (selected && typeof selected === 'string') {
        setSettings({ ...settings, audio_path: selected })
      }
    } catch (error) {
      console.error('选择路径失败:', error)
      alert('选择路径失败: ' + error)
    }
  }

  return (
    <div className="h-full overflow-auto bg-gray-50">
      <div className="max-w-4xl mx-auto p-6">
        <h2 className="text-2xl font-bold text-gray-800 mb-6">应用设置</h2>

        {/* 常规设置 */}
        <div className="bg-white rounded-lg p-6 shadow-sm border border-gray-200 mb-6">
          <h3 className="text-lg font-semibold text-gray-800 mb-4">常规</h3>

          <div className="space-y-4">
            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium text-gray-800">开机自启动</p>
                <p className="text-sm text-gray-500">系统启动时自动运行应用</p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={settings.auto_start}
                  onChange={(e) =>
                    setSettings({ ...settings, auto_start: e.target.checked })
                  }
                  className="sr-only peer"
                />
                <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
              </label>
            </div>

            <div className="flex items-center justify-between">
              <div>
                <p className="font-medium text-gray-800">最小化到系统托盘</p>
                <p className="text-sm text-gray-500">关闭窗口时最小化到托盘</p>
              </div>
              <label className="relative inline-flex items-center cursor-pointer">
                <input
                  type="checkbox"
                  checked={settings.minimize_to_tray}
                  onChange={(e) =>
                    setSettings({ ...settings, minimize_to_tray: e.target.checked })
                  }
                  className="sr-only peer"
                />
                <div className="w-11 h-6 bg-gray-200 peer-focus:outline-none peer-focus:ring-4 peer-focus:ring-blue-300 rounded-full peer peer-checked:after:translate-x-full peer-checked:after:border-white after:content-[''] after:absolute after:top-[2px] after:left-[2px] after:bg-white after:border-gray-300 after:border after:rounded-full after:h-5 after:w-5 after:transition-all peer-checked:bg-blue-600"></div>
              </label>
            </div>
          </div>
        </div>

        {/* 音频设置 */}
        <div className="bg-white rounded-lg p-6 shadow-sm border border-gray-200 mb-6">
          <h3 className="text-lg font-semibold text-gray-800 mb-4">音频</h3>

          <div className="space-y-4">
            <div>
              <div className="flex items-center gap-2 mb-2">
                <Volume2 size={20} className="text-gray-600" />
                <label className="font-medium text-gray-800">
                  默认音量: {settings.default_volume}%
                </label>
              </div>
              <input
                type="range"
                min="0"
                max="100"
                value={settings.default_volume}
                onChange={(e) =>
                  setSettings({ ...settings, default_volume: parseInt(e.target.value) })
                }
                className="w-full"
              />
            </div>

            <div>
              <div className="flex items-center gap-2 mb-2">
                <FolderOpen size={20} className="text-gray-600" />
                <label className="font-medium text-gray-800">音频存储路径</label>
              </div>
              <div className="flex gap-2">
                <input
                  type="text"
                  value={settings.audio_path || '使用默认路径'}
                  disabled
                  className="flex-1 px-4 py-2 border border-gray-300 rounded-lg bg-gray-50 text-gray-600"
                />
                <button
                  onClick={handleChangeAudioPath}
                  className="px-4 py-2 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors"
                >
                  更改
                </button>
              </div>
              <p className="text-sm text-gray-500 mt-1">
                当前存储位置：{settings.audio_path || '应用数据目录/audio'}
              </p>
            </div>
          </div>
        </div>

        {/* 外观设置 */}
        <div className="bg-white rounded-lg p-6 shadow-sm border border-gray-200 mb-6">
          <h3 className="text-lg font-semibold text-gray-800 mb-4">外观</h3>

          <div>
            <label className="font-medium text-gray-800 mb-3 block">主题</label>
            <div className="grid grid-cols-3 gap-4">
              <button
                onClick={() => saveSettings({ ...settings, theme: 'light' })}
                className={`flex items-center justify-center gap-2 p-4 rounded-lg border-2 transition-all ${
                  settings.theme === 'light'
                    ? 'border-blue-600 bg-blue-50'
                    : 'border-gray-200 hover:border-gray-300 text-gray-700'
                }`}
              >
                <Sun size={20} />
                <span>浅色</span>
              </button>
              <button
                onClick={() => saveSettings({ ...settings, theme: 'dark' })}
                className={`flex items-center justify-center gap-2 p-4 rounded-lg border-2 transition-all ${
                  settings.theme === 'dark'
                    ? 'border-blue-600 bg-blue-50'
                    : 'border-gray-200 hover:border-gray-300 text-gray-700'
                }`}
              >
                <Moon size={20} />
                <span>深色</span>
              </button>
              <button
                onClick={() => saveSettings({ ...settings, theme: 'auto' })}
                className={`flex items-center justify-center gap-2 p-4 rounded-lg border-2 transition-all ${
                  settings.theme === 'auto'
                    ? 'border-blue-600 bg-blue-50'
                    : 'border-gray-200 hover:border-gray-300 text-gray-700'
                }`}
              >
                <div className="flex">
                  <Sun size={16} />
                  <Moon size={16} />
                </div>
                <span>跟随系统</span>
              </button>
            </div>
          </div>
        </div>

        {/* 数据管理 */}
        <div className="bg-white rounded-lg p-6 shadow-sm border border-gray-200 mb-6">
          <h3 className="text-lg font-semibold text-gray-800 mb-4">数据管理</h3>

          <div className="space-y-3">
            <button
              onClick={handleExportConfig}
              className="w-full flex items-center justify-center gap-2 p-3 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
            >
              <Download size={20} />
              <span>导出配置和数据</span>
            </button>

            <button
              onClick={handleImportConfig}
              className="w-full flex items-center justify-center gap-2 p-3 bg-gray-100 text-gray-700 rounded-lg hover:bg-gray-200 transition-colors"
            >
              <Upload size={20} />
              <span>导入配置和数据</span>
            </button>

            <div className="pt-3 border-t border-gray-200">
              <p className="text-sm text-gray-600 mb-2">数据统计</p>
              <div className="grid grid-cols-2 gap-4 text-sm">
                <div className="bg-gray-50 p-3 rounded-lg">
                  <p className="text-gray-600">数据库大小</p>
                  <p className="font-semibold text-gray-800">2.3 MB</p>
                </div>
                <div className="bg-gray-50 p-3 rounded-lg">
                  <p className="text-gray-600">音频文件</p>
                  <p className="font-semibold text-gray-800">145.6 MB</p>
                </div>
              </div>
            </div>
          </div>
        </div>

        {/* 保存按钮 */}
        <div className="flex items-center justify-end gap-3">
          {saved && (
            <span className="text-green-600 text-sm">✓ 设置已保存</span>
          )}
          <button
            onClick={handleSave}
            className="flex items-center gap-2 px-6 py-2.5 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            <Save size={18} />
            <span>保存设置</span>
          </button>
        </div>

        {/* 关于信息 */}
        <div className="mt-6 pt-6 border-t border-gray-200 text-center text-sm text-gray-500">
          <p>磨耳朵 v0.1.0</p>
          <p className="mt-1">开源免费跨平台定时音频播放软件</p>
        </div>
      </div>
    </div>
  )
}
