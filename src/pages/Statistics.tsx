import { useState, useEffect } from 'react'
import { Clock, Music, Play, TrendingUp } from 'lucide-react'
import { invoke } from '@tauri-apps/api/tauri'

interface Stats {
  total_audio_count: number
  total_play_count: number
  total_play_duration: number
  this_week_play_count: number
  this_month_play_count: number
}

interface TopAudio {
  id: number
  name: string
  play_count: number
  duration: number
}

export default function Statistics() {
  const [stats, setStats] = useState<Stats | null>(null)
  const [topAudios, setTopAudios] = useState<TopAudio[]>([])
  const [recentHistory, setRecentHistory] = useState<any[]>([])

  useEffect(() => {
    loadStatistics()
  }, [])

  const loadStatistics = async () => {
    try {
      // 从后端获取真实统计数据
      const statsData = await invoke<Stats>('get_statistics')
      setStats(statsData)

      // 获取热门音频
      const topAudiosData = await invoke<TopAudio[]>('get_top_audios', { limit: 5 })
      setTopAudios(topAudiosData)

      // 获取最近28天的活动
      const activityData = await invoke<any[]>('get_daily_activity', { days: 28 })
      setRecentHistory(activityData)
    } catch (error) {
      console.error('加载统计数据失败:', error)
      // 使用默认值
      setStats({
        total_audio_count: 0,
        total_play_count: 0,
        total_play_duration: 0,
        this_week_play_count: 0,
        this_month_play_count: 0,
      })
      setTopAudios([])
    }
  }

  const formatDuration = (seconds: number) => {
    const hours = Math.floor(seconds / 3600)
    const mins = Math.floor((seconds % 3600) / 60)
    if (hours > 0) {
      return `${hours}小时${mins}分钟`
    }
    return `${mins}分钟`
  }

  if (!stats) {
    return (
      <div className="p-6 flex items-center justify-center h-full">
        <p className="text-gray-500">加载中...</p>
      </div>
    )
  }

  return (
    <div className="h-full overflow-auto bg-gray-50">
      <div className="p-6">
        <h2 className="text-2xl font-bold text-gray-800 mb-6">学习统计</h2>

        {/* 统计卡片 */}
        <div className="grid grid-cols-1 md:grid-cols-2 lg:grid-cols-4 gap-4 mb-6">
          <div className="bg-white rounded-lg p-6 shadow-sm border border-gray-200">
            <div className="flex items-center justify-between mb-2">
              <span className="text-gray-600">音频总数</span>
              <Music className="text-blue-600" size={24} />
            </div>
            <p className="text-3xl font-bold text-gray-800">{stats.total_audio_count}</p>
            <p className="text-sm text-gray-500 mt-1">个音频文件</p>
          </div>

          <div className="bg-white rounded-lg p-6 shadow-sm border border-gray-200">
            <div className="flex items-center justify-between mb-2">
              <span className="text-gray-600">累计播放</span>
              <Play className="text-green-600" size={24} />
            </div>
            <p className="text-3xl font-bold text-gray-800">{stats.total_play_count}</p>
            <p className="text-sm text-gray-500 mt-1">次播放记录</p>
          </div>

          <div className="bg-white rounded-lg p-6 shadow-sm border border-gray-200">
            <div className="flex items-center justify-between mb-2">
              <span className="text-gray-600">学习时长</span>
              <Clock className="text-purple-600" size={24} />
            </div>
            <p className="text-3xl font-bold text-gray-800">
              {formatDuration(stats.total_play_duration)}
            </p>
            <p className="text-sm text-gray-500 mt-1">累计学习时长</p>
          </div>

          <div className="bg-white rounded-lg p-6 shadow-sm border border-gray-200">
            <div className="flex items-center justify-between mb-2">
              <span className="text-gray-600">本周播放</span>
              <TrendingUp className="text-orange-600" size={24} />
            </div>
            <p className="text-3xl font-bold text-gray-800">{stats.this_week_play_count}</p>
            <p className="text-sm text-gray-500 mt-1">本周{stats.this_week_play_count}次</p>
          </div>
        </div>

        {/* 热门音频排行 */}
        <div className="bg-white rounded-lg p-6 shadow-sm border border-gray-200 mb-6">
          <h3 className="text-lg font-semibold text-gray-800 mb-4">热门音频排行</h3>
          <div className="space-y-3">
            {topAudios.map((audio, index) => (
              <div
                key={audio.id}
                className="flex items-center gap-4 p-3 rounded-lg hover:bg-gray-50 transition-colors"
              >
                <div
                  className={`flex-shrink-0 w-8 h-8 rounded-full flex items-center justify-center font-bold text-white ${
                    index === 0
                      ? 'bg-yellow-500'
                      : index === 1
                      ? 'bg-gray-400'
                      : index === 2
                      ? 'bg-orange-600'
                      : 'bg-gray-300'
                  }`}
                >
                  {index + 1}
                </div>
                <div className="flex-1 min-w-0">
                  <p className="font-medium text-gray-800 truncate">{audio.name}</p>
                  <p className="text-sm text-gray-500">
                    播放 {audio.play_count} 次 · {formatDuration(audio.duration)}
                  </p>
                </div>
                <div className="flex-shrink-0">
                  <div className="w-32 h-2 bg-gray-200 rounded-full overflow-hidden">
                    <div
                      className="h-full bg-blue-600"
                      style={{
                        width: `${(audio.play_count / topAudios[0].play_count) * 100}%`,
                      }}
                    />
                  </div>
                </div>
              </div>
            ))}
          </div>
        </div>

        {/* 学习日历 */}
        <div className="bg-white rounded-lg p-6 shadow-sm border border-gray-200">
          <h3 className="text-lg font-semibold text-gray-800 mb-4">学习记录（最近28天）</h3>
          <div className="grid grid-cols-7 gap-2">
            {[...Array(28)].map((_, i) => {
              const daysAgo = 27 - i
              const date = new Date()
              date.setDate(date.getDate() - daysAgo)
              const dateStr = date.toISOString().split('T')[0]

              // 查找该日期的活动数据
              const activity = recentHistory.find(h => h.date === dateStr)
              const playCount = activity?.play_count || 0

              // 根据播放次数确定颜色深度
              const intensity = playCount === 0 ? 0 :
                                playCount <= 2 ? 1 :
                                playCount <= 5 ? 2 :
                                playCount <= 10 ? 3 : 4

              return (
                <div
                  key={i}
                  className={`aspect-square rounded ${
                    intensity === 0
                      ? 'bg-gray-100'
                      : intensity === 1
                      ? 'bg-blue-100'
                      : intensity === 2
                      ? 'bg-blue-200'
                      : intensity === 3
                      ? 'bg-blue-400'
                      : 'bg-blue-600'
                  }`}
                  title={`${dateStr}: ${playCount}次播放`}
                />
              )
            })}
          </div>
          <div className="flex items-center gap-2 mt-4 text-sm text-gray-600">
            <span>少</span>
            <div className="w-4 h-4 bg-gray-100 rounded" />
            <div className="w-4 h-4 bg-blue-200 rounded" />
            <div className="w-4 h-4 bg-blue-400 rounded" />
            <div className="w-4 h-4 bg-blue-600 rounded" />
            <span>多</span>
          </div>
        </div>
      </div>
    </div>
  )
}
