import { useState } from 'react'
import { BrowserRouter as Router, Routes, Route, Link } from 'react-router-dom'
import { Music, List, Clock, BarChart3, Settings, HelpCircle } from 'lucide-react'
import AudioLibrary from './pages/AudioLibrary'
import Playlists from './pages/Playlists'
import Tasks from './pages/Tasks'
import Statistics from './pages/Statistics'
import SettingsPage from './pages/Settings'
import Help from './pages/Help'
import PlayController from './components/PlayController'
import { PlayerProvider } from './contexts/PlayerContext'
import { useTheme } from './hooks/useTheme'

function App() {
  const [activeTab, setActiveTab] = useState('audio')

  // 在应用启动时加载并应用主题
  useTheme()

  const menuItems = [
    { id: 'audio', label: '音频库', icon: Music, path: '/' },
    { id: 'playlists', label: '播放列表', icon: List, path: '/playlists' },
    { id: 'tasks', label: '定时任务', icon: Clock, path: '/tasks' },
    { id: 'statistics', label: '统计', icon: BarChart3, path: '/statistics' },
    { id: 'settings', label: '设置', icon: Settings, path: '/settings' },
    { id: 'help', label: '帮助', icon: HelpCircle, path: '/help' },
  ]

  return (
    <PlayerProvider>
      <Router>
        <div className="flex flex-col h-screen bg-gray-50">
          <div className="flex flex-1 overflow-hidden">
            {/* 侧边栏 */}
            <aside className="w-56 bg-white border-r border-gray-200 flex flex-col">
              <div className="p-4 border-b border-gray-200">
                <h1 className="text-xl font-bold text-gray-800">磨耳朵</h1>
                <p className="text-xs text-gray-500 mt-1">定时音频播放软件</p>
              </div>

              <nav className="flex-1 p-3">
                {menuItems.map((item) => (
                  <Link
                    key={item.id}
                    to={item.path}
                    onClick={() => setActiveTab(item.id)}
                    className={`flex items-center gap-3 px-3 py-2.5 rounded-lg mb-1 transition-colors ${
                      activeTab === item.id
                        ? 'bg-blue-50 text-blue-600'
                        : 'text-gray-700 hover:bg-gray-100'
                    }`}
                  >
                    <item.icon size={20} />
                    <span className="text-sm font-medium">{item.label}</span>
                  </Link>
                ))}
              </nav>
            </aside>

            {/* 主内容区 */}
            <main className="flex-1 overflow-auto">
              <Routes>
                <Route path="/" element={<AudioLibrary />} />
                <Route path="/playlists" element={<Playlists />} />
                <Route path="/tasks" element={<Tasks />} />
                <Route path="/statistics" element={<Statistics />} />
                <Route path="/settings" element={<SettingsPage />} />
                <Route path="/help" element={<Help />} />
              </Routes>
            </main>
          </div>

          {/* 全局播放控制器 */}
          <PlayController />
        </div>
      </Router>
    </PlayerProvider>
  )
}

export default App
