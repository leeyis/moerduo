import { useState, useEffect } from 'react'
import { ChevronLeft, ChevronRight, Calendar as CalendarIcon } from 'lucide-react'
import { invoke } from '@tauri-apps/api/tauri'

interface MonthlyPlayback {
  date: string
  play_count: number
  playlists: PlaylistPlayInfo[]
}

interface PlaylistPlayInfo {
  playlist_name: string | null
  audio_count: number
}

export default function Statistics() {
  const [currentDate, setCurrentDate] = useState(new Date())
  const [monthlyData, setMonthlyData] = useState<MonthlyPlayback[]>([])
  const [selectedDate, setSelectedDate] = useState<MonthlyPlayback | null>(null)

  useEffect(() => {
    loadMonthlyData()
  }, [currentDate])

  const loadMonthlyData = async () => {
    try {
      const year = currentDate.getFullYear()
      const month = currentDate.getMonth() + 1
      const data = await invoke<MonthlyPlayback[]>('get_monthly_playback', { year, month })
      setMonthlyData(data)
    } catch (error) {
      console.error('加载播放记录失败:', error)
      setMonthlyData([])
    }
  }

  const getDaysInMonth = (date: Date) => {
    return new Date(date.getFullYear(), date.getMonth() + 1, 0).getDate()
  }

  const getFirstDayOfMonth = (date: Date) => {
    return new Date(date.getFullYear(), date.getMonth(), 1).getDay()
  }

  const prevMonth = () => {
    setCurrentDate(new Date(currentDate.getFullYear(), currentDate.getMonth() - 1, 1))
    setSelectedDate(null)
  }

  const nextMonth = () => {
    setCurrentDate(new Date(currentDate.getFullYear(), currentDate.getMonth() + 1, 1))
    setSelectedDate(null)
  }

  const getPlaybackForDate = (day: number): MonthlyPlayback | null => {
    const dateStr = `${currentDate.getFullYear()}-${String(currentDate.getMonth() + 1).padStart(2, '0')}-${String(day).padStart(2, '0')}`
    return monthlyData.find(d => d.date === dateStr) || null
  }

  const renderCalendar = () => {
    const daysInMonth = getDaysInMonth(currentDate)
    const firstDay = getFirstDayOfMonth(currentDate)
    const days = []

    // 填充空白天数
    for (let i = 0; i < firstDay; i++) {
      days.push(<div key={`empty-${i}`} className="aspect-square" />)
    }

    // 填充实际天数
    for (let day = 1; day <= daysInMonth; day++) {
      const playback = getPlaybackForDate(day)
      const isToday =
        day === new Date().getDate() &&
        currentDate.getMonth() === new Date().getMonth() &&
        currentDate.getFullYear() === new Date().getFullYear()

      days.push(
        <div
          key={day}
          onClick={() => playback && setSelectedDate(playback)}
          className={`aspect-square border rounded-lg p-2 cursor-pointer transition-all ${
            playback
              ? 'bg-blue-50 hover:bg-blue-100 border-blue-200'
              : 'bg-gray-50 hover:bg-gray-100 border-gray-200'
          } ${isToday ? 'ring-2 ring-blue-500' : ''} ${
            selectedDate && selectedDate.date === getPlaybackForDate(day)?.date
              ? 'ring-2 ring-blue-600 bg-blue-100'
              : ''
          }`}
        >
          <div className="flex flex-col h-full">
            <div className="text-sm font-medium text-gray-700 mb-1">{day}</div>
            {playback && (
              <div className="flex-1 overflow-hidden">
                <div className="text-xs font-bold text-blue-600 mb-1">
                  {playback.play_count}次
                </div>
                <div className="text-xs text-gray-600 space-y-0.5">
                  {playback.playlists.slice(0, 2).map((pl, idx) => (
                    <div key={idx} className="truncate">
                      {pl.playlist_name || '单独播放'}: {pl.audio_count}
                    </div>
                  ))}
                  {playback.playlists.length > 2 && (
                    <div className="text-gray-400">+{playback.playlists.length - 2}项</div>
                  )}
                </div>
              </div>
            )}
          </div>
        </div>
      )
    }

    return days
  }

  return (
    <div className="h-full flex flex-col bg-gray-50">
      <div className="p-6">
        <div className="flex items-center justify-between mb-6">
          <h2 className="text-2xl font-bold text-gray-800">播放记录</h2>
          <div className="flex items-center gap-3">
            <button
              onClick={prevMonth}
              className="p-2 hover:bg-gray-200 rounded-lg transition-colors"
            >
              <ChevronLeft size={20} />
            </button>
            <div className="flex items-center gap-2 px-4 py-2 bg-white rounded-lg border border-gray-200">
              <CalendarIcon size={18} className="text-blue-600" />
              <span className="font-medium">
                {currentDate.getFullYear()}年{currentDate.getMonth() + 1}月
              </span>
            </div>
            <button
              onClick={nextMonth}
              className="p-2 hover:bg-gray-200 rounded-lg transition-colors"
            >
              <ChevronRight size={20} />
            </button>
          </div>
        </div>

        <div className="bg-white rounded-lg p-6 shadow-sm border border-gray-200">
          {/* 星期标题 */}
          <div className="grid grid-cols-7 gap-2 mb-2">
            {['日', '一', '二', '三', '四', '五', '六'].map((day) => (
              <div
                key={day}
                className="text-center text-sm font-medium text-gray-600 py-2"
              >
                {day}
              </div>
            ))}
          </div>

          {/* 日历格子 */}
          <div className="grid grid-cols-7 gap-2">{renderCalendar()}</div>
        </div>

        {/* 详情面板 */}
        {selectedDate && (
          <div className="mt-6 bg-white rounded-lg p-6 shadow-sm border border-gray-200">
            <h3 className="text-lg font-bold text-gray-800 mb-4">
              {selectedDate.date} 播放详情
            </h3>
            <div className="space-y-3">
              <div className="flex items-center justify-between p-3 bg-blue-50 rounded-lg">
                <span className="text-gray-700 font-medium">总播放次数</span>
                <span className="text-xl font-bold text-blue-600">
                  {selectedDate.play_count} 次
                </span>
              </div>

              <div className="space-y-2">
                <h4 className="font-medium text-gray-700">播放详情：</h4>
                {selectedDate.playlists.map((playlist, idx) => (
                  <div
                    key={idx}
                    className="flex items-center justify-between p-3 bg-gray-50 rounded-lg"
                  >
                    <div>
                      <div className="font-medium text-gray-800">
                        {playlist.playlist_name || '单独播放'}
                      </div>
                    </div>
                    <div className="text-right">
                      <div className="text-sm text-gray-600">{playlist.audio_count} 首音频</div>
                    </div>
                  </div>
                ))}
              </div>
            </div>
          </div>
        )}
      </div>
    </div>
  )
}
