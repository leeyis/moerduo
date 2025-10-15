import { useState, useEffect } from 'react'
import { Plus, Trash2, Edit2, Clock, Power } from 'lucide-react'
import { invoke } from '@tauri-apps/api/tauri'

interface Task {
  id: number
  name: string
  hour: number
  minute: number
  repeat_mode: string
  custom_days: string | null
  playlist_id: number
  playlist_name: string
  volume: number
  fade_in_duration: number
  duration_minutes: number | null
  is_enabled: boolean
  priority: number
  created_date: string
}

interface Playlist {
  id: number
  name: string
}

interface TaskConflict {
  task_id: number
  task_name: string
  hour: number
  minute: number
}

export default function Tasks() {
  const [tasks, setTasks] = useState<Task[]>([])
  const [playlists, setPlaylists] = useState<Playlist[]>([])
  const [showDialog, setShowDialog] = useState(false)
  const [editingTask, setEditingTask] = useState<Task | null>(null)
  const [conflicts, setConflicts] = useState<TaskConflict[]>([])
  const [showConflictDialog, setShowConflictDialog] = useState(false)
  const [showDeleteConfirm, setShowDeleteConfirm] = useState(false)
  const [taskToDelete, setTaskToDelete] = useState<number | null>(null)

  const [formData, setFormData] = useState({
    name: '',
    hour: 7,
    minute: 0,
    repeat_mode: 'weekday',
    custom_days: [] as number[],
    playlist_id: 0,
    volume: 50,
    fade_in_duration: 30,
    duration_minutes: null as number | null,
    priority: 0,
  })

  useEffect(() => {
    loadTasks()
    loadPlaylists()
  }, [])

  const loadTasks = async () => {
    try {
      const taskList = await invoke<Task[]>('get_scheduled_tasks')
      setTasks(taskList)
    } catch (error) {
      console.error('加载定时任务失败:', error)
    }
  }

  const loadPlaylists = async () => {
    try {
      const lists = await invoke<Playlist[]>('get_playlists')
      setPlaylists(lists)
      if (lists.length > 0 && formData.playlist_id === 0) {
        setFormData(prev => ({ ...prev, playlist_id: lists[0].id }))
      }
    } catch (error) {
      console.error('加载播放列表失败:', error)
    }
  }

  const handleSaveTask = async () => {
    if (!formData.name.trim()) {
      alert('请输入任务名称')
      return
    }

    if (formData.playlist_id === 0) {
      alert('请选择播放列表')
      return
    }

    try {
      const customDaysStr = formData.repeat_mode === 'custom'
        ? JSON.stringify(formData.custom_days)
        : null

      // 检查任务冲突
      const conflictList = await invoke<TaskConflict[]>('check_task_conflicts', {
        taskId: editingTask?.id || null,
        hour: formData.hour,
        minute: formData.minute,
        repeatMode: formData.repeat_mode,
        customDays: customDaysStr,
        durationMinutes: formData.duration_minutes,
        playlistId: formData.playlist_id,
      })

      if (conflictList.length > 0) {
        setConflicts(conflictList)
        setShowConflictDialog(true)
        return
      }

      // 没有冲突，保存任务
      if (editingTask) {
        await invoke('update_scheduled_task', {
          id: editingTask.id,
          name: formData.name,
          hour: formData.hour,
          minute: formData.minute,
          repeatMode: formData.repeat_mode,
          customDays: customDaysStr,
          playlistId: formData.playlist_id,
          volume: formData.volume,
          fadeInDuration: formData.fade_in_duration,
          durationMinutes: formData.duration_minutes,
          priority: formData.priority,
        })
      } else {
        await invoke('create_scheduled_task', {
          name: formData.name,
          hour: formData.hour,
          minute: formData.minute,
          repeatMode: formData.repeat_mode,
          customDays: customDaysStr,
          playlistId: formData.playlist_id,
          volume: formData.volume,
          fadeInDuration: formData.fade_in_duration,
          durationMinutes: formData.duration_minutes,
          priority: formData.priority,
        })
      }

      resetForm()
      loadTasks()
    } catch (error) {
      console.error('保存任务失败:', error)
      alert('保存任务失败: ' + error)
    }
  }

  const handleToggleTask = async (id: number, enabled: boolean) => {
    try {
      await invoke('toggle_scheduled_task', { id, enabled })
      loadTasks()
    } catch (error) {
      console.error('切换任务状态失败:', error)
    }
  }

  const handleDeleteTask = (id: number) => {
    setTaskToDelete(id)
    setShowDeleteConfirm(true)
  }

  const confirmDeleteTask = async () => {
    if (taskToDelete === null) return

    try {
      await invoke('delete_scheduled_task', { id: taskToDelete })
      loadTasks()
      setShowDeleteConfirm(false)
      setTaskToDelete(null)
    } catch (error) {
      console.error('删除任务失败:', error)
      alert('删除任务失败: ' + error)
    }
  }

  const cancelDeleteTask = () => {
    setShowDeleteConfirm(false)
    setTaskToDelete(null)
  }

  const handleEditTask = (task: Task) => {
    setEditingTask(task)
    setFormData({
      name: task.name,
      hour: task.hour,
      minute: task.minute,
      repeat_mode: task.repeat_mode,
      custom_days: task.custom_days ? JSON.parse(task.custom_days) : [],
      playlist_id: task.playlist_id,
      volume: task.volume,
      fade_in_duration: task.fade_in_duration,
      duration_minutes: task.duration_minutes,
      priority: task.priority,
    })
    setShowDialog(true)
  }

  const resetForm = () => {
    setFormData({
      name: '',
      hour: 7,
      minute: 0,
      repeat_mode: 'weekday',
      custom_days: [],
      playlist_id: playlists.length > 0 ? playlists[0].id : 0,
      volume: 50,
      fade_in_duration: 30,
      duration_minutes: null,
      priority: 0,
    })
    setEditingTask(null)
    setShowDialog(false)
  }

  const getRepeatModeName = (mode: string, customDays: string | null) => {
    switch (mode) {
      case 'daily':
        return '每天'
      case 'weekday':
        return '工作日'
      case 'weekend':
        return '周末'
      case 'once':
        return '仅一次'
      case 'custom':
        if (!customDays) return '自定义'
        const days = JSON.parse(customDays)
        const dayNames = ['日', '一', '二', '三', '四', '五', '六']
        return '周' + days.map((d: number) => dayNames[d]).join('、')
      default:
        return mode
    }
  }

  const formatTime = (hour: number, minute: number) => {
    return `${hour.toString().padStart(2, '0')}:${minute.toString().padStart(2, '0')}`
  }

  const dayNames = ['周日', '周一', '周二', '周三', '周四', '周五', '周六']

  return (
    <div className="h-full flex flex-col bg-white">
      <div className="border-b border-gray-200 p-6">
        <div className="flex items-center justify-between mb-4">
          <h2 className="text-2xl font-bold text-gray-800">定时任务</h2>
          <button
            onClick={() => setShowDialog(true)}
            className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            <Plus size={18} />
            <span>新建任务</span>
          </button>
        </div>
        <p className="text-gray-600">共 {tasks.length} 个任务，{tasks.filter(t => t.is_enabled).length} 个已启用</p>
      </div>

      <div className="flex-1 overflow-auto p-6">
        {tasks.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-gray-400">
            <Clock size={64} className="mb-4" />
            <p className="text-lg">还没有定时任务</p>
            <p className="text-sm mt-2">点击"新建任务"创建第一个定时播放任务</p>
          </div>
        ) : (
          <div className="grid grid-cols-1 lg:grid-cols-2 gap-4">
            {tasks.map((task) => (
              <div
                key={task.id}
                className={`border rounded-lg p-4 transition-all ${
                  task.is_enabled
                    ? 'border-blue-200 bg-blue-50'
                    : 'border-gray-200 bg-white'
                }`}
              >
                <div className="flex items-start justify-between mb-3">
                  <div className="flex-1">
                    <h3 className="text-lg font-semibold text-gray-800 mb-1">
                      {task.name}
                    </h3>
                    <p className="text-3xl font-bold text-blue-600">
                      {formatTime(task.hour, task.minute)}
                    </p>
                  </div>
                  <div className="flex gap-2">
                    <button
                      onClick={() => handleToggleTask(task.id, !task.is_enabled)}
                      className={`p-2 rounded-lg transition-colors ${
                        task.is_enabled
                          ? 'bg-blue-600 text-white hover:bg-blue-700'
                          : 'bg-gray-200 text-gray-600 hover:bg-gray-300'
                      }`}
                      title={task.is_enabled ? '已启用' : '已禁用'}
                    >
                      <Power size={16} />
                    </button>
                  </div>
                </div>

                <div className="space-y-2 text-sm text-gray-600 mb-3">
                  <div className="flex items-center gap-2">
                    <Clock size={14} />
                    <span>{getRepeatModeName(task.repeat_mode, task.custom_days)}</span>
                  </div>
                  <div>
                    <span className="text-gray-500">播放列表:</span>{' '}
                    <span className="font-medium">{task.playlist_name}</span>
                  </div>
                  <div className="flex gap-4">
                    <span>
                      <span className="text-gray-500">音量:</span> {task.volume}%
                    </span>
                    <span>
                      <span className="text-gray-500">渐强:</span> {task.fade_in_duration}秒
                    </span>
                    {task.duration_minutes && (
                      <span>
                        <span className="text-gray-500">时长:</span> {task.duration_minutes}分钟
                      </span>
                    )}
                  </div>
                </div>

                <div className="flex gap-2">
                  <button
                    onClick={() => handleEditTask(task)}
                    className="flex-1 flex items-center justify-center gap-2 px-3 py-2 text-gray-700 bg-gray-100 rounded-lg hover:bg-gray-200 transition-colors"
                  >
                    <Edit2 size={14} />
                    <span>编辑</span>
                  </button>
                  <button
                    onClick={() => handleDeleteTask(task.id)}
                    className="flex items-center gap-2 px-3 py-2 text-red-600 bg-red-50 rounded-lg hover:bg-red-100 transition-colors"
                  >
                    <Trash2 size={14} />
                    <span>删除</span>
                  </button>
                </div>
              </div>
            ))}
          </div>
        )}
      </div>

      {/* 新建/编辑任务对话框 */}
      {showDialog && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-[600px] max-h-[90vh] overflow-y-auto">
            <h3 className="text-xl font-bold mb-4">
              {editingTask ? '编辑任务' : '新建任务'}
            </h3>

            <div className="space-y-4">
              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  任务名称
                </label>
                <input
                  type="text"
                  value={formData.name}
                  onChange={(e) => setFormData({ ...formData, name: e.target.value })}
                  placeholder="例如：早安英语"
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
              </div>

              <div className="grid grid-cols-2 gap-4">
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    小时
                  </label>
                  <input
                    type="number"
                    min="0"
                    max="23"
                    value={formData.hour}
                    onChange={(e) =>
                      setFormData({ ...formData, hour: parseInt(e.target.value) })
                    }
                    className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-1">
                    分钟
                  </label>
                  <input
                    type="number"
                    min="0"
                    max="59"
                    value={formData.minute}
                    onChange={(e) =>
                      setFormData({ ...formData, minute: parseInt(e.target.value) })
                    }
                    className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                  />
                </div>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  重复模式
                </label>
                <select
                  value={formData.repeat_mode}
                  onChange={(e) =>
                    setFormData({ ...formData, repeat_mode: e.target.value })
                  }
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  <option value="daily">每天</option>
                  <option value="weekday">工作日（周一至周五）</option>
                  <option value="weekend">周末（周六、周日）</option>
                  <option value="custom">自定义</option>
                  <option value="once">仅一次</option>
                </select>
              </div>

              {formData.repeat_mode === 'custom' && (
                <div>
                  <label className="block text-sm font-medium text-gray-700 mb-2">
                    选择星期
                  </label>
                  <div className="flex gap-2">
                    {dayNames.map((day, index) => (
                      <button
                        key={index}
                        onClick={() => {
                          const days = [...formData.custom_days]
                          const idx = days.indexOf(index)
                          if (idx > -1) {
                            days.splice(idx, 1)
                          } else {
                            days.push(index)
                          }
                          setFormData({ ...formData, custom_days: days.sort() })
                        }}
                        className={`flex-1 py-2 rounded-lg transition-colors ${
                          formData.custom_days.includes(index)
                            ? 'bg-blue-600 text-white'
                            : 'bg-gray-100 text-gray-700 hover:bg-gray-200'
                        }`}
                      >
                        {day}
                      </button>
                    ))}
                  </div>
                </div>
              )}

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  播放列表
                </label>
                <select
                  value={formData.playlist_id}
                  onChange={(e) =>
                    setFormData({ ...formData, playlist_id: parseInt(e.target.value) })
                  }
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                >
                  {playlists.map((playlist) => (
                    <option key={playlist.id} value={playlist.id}>
                      {playlist.name}
                    </option>
                  ))}
                </select>
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  音量: {formData.volume}%
                </label>
                <input
                  type="range"
                  min="0"
                  max="100"
                  value={formData.volume}
                  onChange={(e) =>
                    setFormData({ ...formData, volume: parseInt(e.target.value) })
                  }
                  className="w-full"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  渐强时长: {formData.fade_in_duration}秒
                </label>
                <input
                  type="range"
                  min="0"
                  max="300"
                  step="10"
                  value={formData.fade_in_duration}
                  onChange={(e) =>
                    setFormData({
                      ...formData,
                      fade_in_duration: parseInt(e.target.value),
                    })
                  }
                  className="w-full"
                />
              </div>

              <div>
                <label className="block text-sm font-medium text-gray-700 mb-1">
                  播放时长（留空表示播放全部）: {formData.duration_minutes ? `${formData.duration_minutes}分钟` : '播放全部'}
                </label>
                <input
                  type="number"
                  min="1"
                  max="480"
                  value={formData.duration_minutes || ''}
                  onChange={(e) =>
                    setFormData({
                      ...formData,
                      duration_minutes: e.target.value ? parseInt(e.target.value) : null,
                    })
                  }
                  placeholder="留空表示播放列表全部音频"
                  className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500"
                />
                <p className="text-xs text-gray-500 mt-1">
                  设置播放时长可以避免长时间播放影响下一个任务
                </p>
              </div>
            </div>

            <div className="flex justify-end gap-2 mt-6">
              <button
                onClick={resetForm}
                className="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
              >
                取消
              </button>
              <button
                onClick={handleSaveTask}
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              >
                {editingTask ? '保存' : '创建'}
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 冲突提示对话框 */}
      {showConflictDialog && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-[500px]">
            <h3 className="text-xl font-bold mb-4 text-red-800">任务时间冲突</h3>
            <div className="mb-6">
              <p className="text-gray-700 mb-4">以下任务与当前任务的时间段有冲突：</p>
              <ul className="space-y-2">
                {conflicts.map((conflict) => (
                  <li key={conflict.task_id} className="bg-red-50 border border-red-200 rounded-lg p-3">
                    <div className="flex items-center justify-between">
                      <span className="font-medium text-red-800">{conflict.task_name}</span>
                      <span className="text-red-600">
                        {formatTime(conflict.hour, conflict.minute)}
                      </span>
                    </div>
                  </li>
                ))}
              </ul>
              <p className="text-gray-600 mt-4 text-sm">
                请修改当前任务的时间或播放时长，或者调整冲突任务的时间设置。
              </p>
            </div>
            <div className="flex justify-end gap-2">
              <button
                onClick={() => setShowConflictDialog(false)}
                className="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
              >
                返回修改
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 删除确认对话框 */}
      {showDeleteConfirm && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-[480px]">
            <h3 className="text-xl font-bold mb-4 text-gray-800">确认删除</h3>
            <p className="text-gray-600 mb-6">确定要删除该定时任务吗？此操作无法撤销。</p>
            <div className="flex justify-end gap-2">
              <button
                onClick={cancelDeleteTask}
                className="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
              >
                取消
              </button>
              <button
                onClick={confirmDeleteTask}
                className="px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
              >
                删除
              </button>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
