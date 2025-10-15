import { useState, useEffect } from 'react'
import { Plus, Trash2, Edit2, List as ListIcon, Shuffle, Repeat, Repeat1, Music } from 'lucide-react'
import { invoke } from '@tauri-apps/api/tauri'

interface Playlist {
  id: number
  name: string
  play_mode: string
  created_date: string
  updated_date: string
}

interface PlaylistItem {
  id: number
  playlist_id: number
  audio_id: number
  sort_order: number
  audio_name: string
  duration: number
}

interface AudioFile {
  id: number
  filename: string
  original_name: string
  file_size: number
  duration: number
  format: string
}

export default function Playlists() {
  const [playlists, setPlaylists] = useState<Playlist[]>([])
  const [selectedPlaylist, setSelectedPlaylist] = useState<number | null>(null)
  const [playlistItems, setPlaylistItems] = useState<PlaylistItem[]>([])
  const [showNewDialog, setShowNewDialog] = useState(false)
  const [newPlaylistName, setNewPlaylistName] = useState('')
  const [showAddAudioDialog, setShowAddAudioDialog] = useState(false)
  const [audioFiles, setAudioFiles] = useState<AudioFile[]>([])
  const [selectedAudios, setSelectedAudios] = useState<Set<number>>(new Set())

  useEffect(() => {
    loadPlaylists()
  }, [])

  useEffect(() => {
    if (selectedPlaylist) {
      loadPlaylistItems(selectedPlaylist)
    }
  }, [selectedPlaylist])

  const loadPlaylists = async () => {
    try {
      const lists = await invoke<Playlist[]>('get_playlists')
      setPlaylists(lists)
    } catch (error) {
      console.error('加载播放列表失败:', error)
    }
  }

  const loadPlaylistItems = async (playlistId: number) => {
    try {
      const items = await invoke<PlaylistItem[]>('get_playlist_items', { playlistId })
      setPlaylistItems(items)
    } catch (error) {
      console.error('加载播放列表项失败:', error)
    }
  }

  const handleCreatePlaylist = async () => {
    if (!newPlaylistName.trim()) return

    try {
      await invoke('create_playlist', { name: newPlaylistName })
      setNewPlaylistName('')
      setShowNewDialog(false)
      loadPlaylists()
    } catch (error) {
      console.error('创建播放列表失败:', error)
    }
  }

  const handleDeletePlaylist = async (id: number) => {
    if (!confirm('确定删除该播放列表吗？')) return

    try {
      await invoke('delete_playlist', { id })
      if (selectedPlaylist === id) {
        setSelectedPlaylist(null)
        setPlaylistItems([])
      }
      loadPlaylists()
    } catch (error) {
      console.error('删除播放列表失败:', error)
    }
  }

  const handleSetPlayMode = async (mode: string) => {
    if (!selectedPlaylist) return

    try {
      await invoke('set_playlist_mode', { playlistId: selectedPlaylist, mode })
      loadPlaylists()
    } catch (error) {
      console.error('设置播放模式失败:', error)
    }
  }

  const loadAudioFiles = async () => {
    try {
      const files = await invoke<AudioFile[]>('get_audio_files')
      setAudioFiles(files)
    } catch (error) {
      console.error('加载音频文件失败:', error)
    }
  }

  const handleOpenAddAudioDialog = () => {
    setShowAddAudioDialog(true)
    setSelectedAudios(new Set())
    loadAudioFiles()
  }

  const handleAddAudiosToPlaylist = async () => {
    if (!selectedPlaylist || selectedAudios.size === 0) return

    try {
      for (const audioId of selectedAudios) {
        await invoke('add_to_playlist', { playlistId: selectedPlaylist, audioId })
      }
      setShowAddAudioDialog(false)
      setSelectedAudios(new Set())
      loadPlaylistItems(selectedPlaylist)
    } catch (error) {
      console.error('添加音频到播放列表失败:', error)
      alert('添加失败: ' + error)
    }
  }

  const handleRemoveFromPlaylist = async (itemId: number) => {
    if (!selectedPlaylist) return

    try {
      await invoke('remove_from_playlist', { itemId })
      loadPlaylistItems(selectedPlaylist)
    } catch (error) {
      console.error('从播放列表移除失败:', error)
    }
  }

  const toggleAudioSelection = (audioId: number) => {
    const newSelection = new Set(selectedAudios)
    if (newSelection.has(audioId)) {
      newSelection.delete(audioId)
    } else {
      newSelection.add(audioId)
    }
    setSelectedAudios(newSelection)
  }

  const getPlayModeIcon = (mode: string) => {
    switch (mode) {
      case 'sequential':
        return <ListIcon size={16} />
      case 'random':
        return <Shuffle size={16} />
      case 'single':
        return <Repeat1 size={16} />
      case 'loop':
        return <Repeat size={16} />
      default:
        return <ListIcon size={16} />
    }
  }

  const getPlayModeName = (mode: string) => {
    switch (mode) {
      case 'sequential':
        return '顺序播放'
      case 'random':
        return '随机播放'
      case 'single':
        return '单曲循环'
      case 'loop':
        return '列表循环'
      default:
        return '顺序播放'
    }
  }

  const formatDuration = (seconds: number) => {
    const mins = Math.floor(seconds / 60)
    const secs = seconds % 60
    return `${mins}:${secs.toString().padStart(2, '0')}`
  }

  const currentPlaylist = playlists.find(p => p.id === selectedPlaylist)

  return (
    <div className="h-full flex bg-white">
      {/* 左侧播放列表 */}
      <div className="w-64 border-r border-gray-200 flex flex-col">
        <div className="p-4 border-b border-gray-200">
          <button
            onClick={() => setShowNewDialog(true)}
            className="w-full flex items-center justify-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            <Plus size={18} />
            <span>新建列表</span>
          </button>
        </div>

        <div className="flex-1 overflow-auto p-3">
          {playlists.map((playlist) => (
            <div
              key={playlist.id}
              onClick={() => setSelectedPlaylist(playlist.id)}
              className={`group p-3 rounded-lg mb-2 cursor-pointer transition-colors ${
                selectedPlaylist === playlist.id
                  ? 'bg-blue-50 border border-blue-200'
                  : 'hover:bg-gray-50 border border-transparent'
              }`}
            >
              <div className="flex items-center justify-between">
                <div className="flex-1 min-w-0">
                  <p className="font-medium text-gray-800 truncate">{playlist.name}</p>
                  <p className="text-xs text-gray-500 mt-1">
                    {getPlayModeName(playlist.play_mode)}
                  </p>
                </div>
                <button
                  onClick={(e) => {
                    e.stopPropagation()
                    handleDeletePlaylist(playlist.id)
                  }}
                  className="opacity-0 group-hover:opacity-100 p-1 text-red-600 hover:bg-red-50 rounded transition-opacity"
                >
                  <Trash2 size={14} />
                </button>
              </div>
            </div>
          ))}

          {playlists.length === 0 && (
            <div className="text-center text-gray-400 mt-8">
              <p className="text-sm">还没有播放列表</p>
            </div>
          )}
        </div>
      </div>

      {/* 右侧内容区 */}
      <div className="flex-1 flex flex-col">
        {currentPlaylist ? (
          <>
            <div className="border-b border-gray-200 p-6">
              <div className="flex items-center justify-between mb-4">
                <h2 className="text-2xl font-bold text-gray-800">{currentPlaylist.name}</h2>
                <div className="flex gap-2">
                  <button
                    onClick={handleOpenAddAudioDialog}
                    className="flex items-center gap-2 px-3 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors"
                  >
                    <Plus size={16} />
                    <span>添加音频</span>
                  </button>
                  {['sequential', 'random', 'single', 'loop'].map((mode) => (
                    <button
                      key={mode}
                      onClick={() => handleSetPlayMode(mode)}
                      className={`p-2 rounded-lg transition-colors ${
                        currentPlaylist.play_mode === mode
                          ? 'bg-blue-600 text-white'
                          : 'bg-gray-100 text-gray-600 hover:bg-gray-200'
                      }`}
                      title={getPlayModeName(mode)}
                    >
                      {getPlayModeIcon(mode)}
                    </button>
                  ))}
                </div>
              </div>

              <p className="text-gray-600">共 {playlistItems.length} 首音频</p>
            </div>

            <div className="flex-1 overflow-auto p-6">
              {playlistItems.length === 0 ? (
                <div className="flex flex-col items-center justify-center h-full text-gray-400">
                  <ListIcon size={64} className="mb-4" />
                  <p className="text-lg">播放列表为空</p>
                  <p className="text-sm mt-2">从音频库添加音频到此列表</p>
                </div>
              ) : (
                <table className="w-full">
                  <thead className="border-b border-gray-200">
                    <tr className="text-left text-sm text-gray-600">
                      <th className="pb-3 w-16">#</th>
                      <th className="pb-3">音频名称</th>
                      <th className="pb-3 w-24">时长</th>
                      <th className="pb-3 w-24">操作</th>
                    </tr>
                  </thead>
                  <tbody>
                    {playlistItems.map((item, index) => (
                      <tr
                        key={item.id}
                        className="border-b border-gray-100 hover:bg-gray-50 transition-colors"
                      >
                        <td className="py-3 text-gray-600">{index + 1}</td>
                        <td className="py-3 text-gray-800">{item.audio_name}</td>
                        <td className="py-3 text-gray-600">{formatDuration(item.duration)}</td>
                        <td className="py-3">
                          <button
                            onClick={() => handleRemoveFromPlaylist(item.id)}
                            className="p-2 text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                          >
                            <Trash2 size={16} />
                          </button>
                        </td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>
          </>
        ) : (
          <div className="flex-1 flex items-center justify-center text-gray-400">
            <div className="text-center">
              <ListIcon size={64} className="mx-auto mb-4" />
              <p className="text-lg">请选择一个播放列表</p>
            </div>
          </div>
        )}
      </div>

      {/* 新建播放列表对话框 */}
      {showNewDialog && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-96">
            <h3 className="text-xl font-bold mb-4">新建播放列表</h3>
            <input
              type="text"
              value={newPlaylistName}
              onChange={(e) => setNewPlaylistName(e.target.value)}
              placeholder="请输入播放列表名称"
              className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 mb-4"
              autoFocus
              onKeyPress={(e) => {
                if (e.key === 'Enter') {
                  handleCreatePlaylist()
                }
              }}
            />
            <div className="flex justify-end gap-2">
              <button
                onClick={() => {
                  setShowNewDialog(false)
                  setNewPlaylistName('')
                }}
                className="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
              >
                取消
              </button>
              <button
                onClick={handleCreatePlaylist}
                className="px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
              >
                创建
              </button>
            </div>
          </div>
        </div>
      )}

      {/* 添加音频对话框 */}
      {showAddAudioDialog && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-[700px] max-h-[80vh] flex flex-col">
            <h3 className="text-xl font-bold mb-4">添加音频到播放列表</h3>

            <div className="flex-1 overflow-auto mb-4 border border-gray-200 rounded-lg">
              {audioFiles.length === 0 ? (
                <div className="flex flex-col items-center justify-center h-full text-gray-400 py-12">
                  <Music size={48} className="mb-2" />
                  <p>还没有音频文件</p>
                  <p className="text-sm mt-1">请先在音频库上传音频</p>
                </div>
              ) : (
                <table className="w-full">
                  <thead className="border-b border-gray-200 bg-gray-50 sticky top-0">
                    <tr className="text-left text-sm text-gray-600">
                      <th className="p-3 w-12">
                        <input
                          type="checkbox"
                          checked={selectedAudios.size === audioFiles.length && audioFiles.length > 0}
                          onChange={(e) => {
                            if (e.target.checked) {
                              setSelectedAudios(new Set(audioFiles.map(f => f.id)))
                            } else {
                              setSelectedAudios(new Set())
                            }
                          }}
                          className="w-4 h-4"
                        />
                      </th>
                      <th className="p-3">文件名</th>
                      <th className="p-3 w-24">时长</th>
                      <th className="p-3 w-20">格式</th>
                    </tr>
                  </thead>
                  <tbody>
                    {audioFiles.map((audio) => (
                      <tr
                        key={audio.id}
                        className="border-b border-gray-100 hover:bg-gray-50 transition-colors cursor-pointer"
                        onClick={() => toggleAudioSelection(audio.id)}
                      >
                        <td className="p-3">
                          <input
                            type="checkbox"
                            checked={selectedAudios.has(audio.id)}
                            onChange={() => toggleAudioSelection(audio.id)}
                            className="w-4 h-4"
                          />
                        </td>
                        <td className="p-3 text-gray-800">{audio.original_name}</td>
                        <td className="p-3 text-gray-600">{formatDuration(audio.duration)}</td>
                        <td className="p-3 text-gray-600 uppercase">{audio.format}</td>
                      </tr>
                    ))}
                  </tbody>
                </table>
              )}
            </div>

            <div className="flex items-center justify-between">
              <span className="text-sm text-gray-600">
                已选择 {selectedAudios.size} 个音频
              </span>
              <div className="flex gap-2">
                <button
                  onClick={() => {
                    setShowAddAudioDialog(false)
                    setSelectedAudios(new Set())
                  }}
                  className="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
                >
                  取消
                </button>
                <button
                  onClick={handleAddAudiosToPlaylist}
                  disabled={selectedAudios.size === 0}
                  className="px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
                >
                  添加 {selectedAudios.size > 0 && `(${selectedAudios.size})`}
                </button>
              </div>
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
