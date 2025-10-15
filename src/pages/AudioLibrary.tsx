import { useState, useEffect } from 'react'
import { Upload, Trash2, Play, Pause, Square, Search, Music, RefreshCw, Mic } from 'lucide-react'
import { invoke } from '@tauri-apps/api/tauri'
import { open } from '@tauri-apps/api/dialog'
import { listen } from '@tauri-apps/api/event'
import { usePlayer } from '../contexts/PlayerContext'
import DeleteConfirmDialog from '../components/DeleteConfirmDialog'

interface AudioFile {
  id: number
  filename: string
  original_name: string
  file_size: number
  duration: number
  format: string
  upload_date: string
}

export default function AudioLibrary() {
  const [audioFiles, setAudioFiles] = useState<AudioFile[]>([])
  const [selectedFiles, setSelectedFiles] = useState<Set<number>>(new Set())
  const [searchTerm, setSearchTerm] = useState('')
  const { isPlaying, currentAudio, playAudio, pauseAudio, stopAudio } = usePlayer()
  const [isDragging, setIsDragging] = useState(false)
  const [isScanning, setIsScanning] = useState(false)
  const [showDeleteDialog, setShowDeleteDialog] = useState(false)
  const [showRecordDialog, setShowRecordDialog] = useState(false)
  const [isRecording, setIsRecording] = useState(false)
  const [recordingFilename, setRecordingFilename] = useState('')

  const handleUpload = async () => {
    try {
      const selected = await open({
        multiple: true,
        filters: [{
          name: 'Audio',
          extensions: ['mp3', 'wav', 'ogg', 'flac', 'm4a']
        }]
      })

      if (selected && Array.isArray(selected)) {
        for (const filePath of selected) {
          await invoke('upload_audio_file', { filePath })
        }
        loadAudioFiles()
      }
    } catch (error) {
      console.error('上传失败:', error)
    }
  }

  const handleScan = async () => {
    if (isScanning) return

    setIsScanning(true)
    try {
      const result = await invoke<{
        found_files: number
        added_files: number
        skipped_files: number
        error_files: number
      }>('scan_audio_directory')

      // 显示扫描结果
      let message = `扫描完成！\n`
      message += `发现文件: ${result.found_files} 个\n`
      message += `新增文件: ${result.added_files} 个\n`
      message += `跳过文件: ${result.skipped_files} 个（已存在）\n`
      if (result.error_files > 0) {
        message += `错误文件: ${result.error_files} 个`
      }

      if (result.added_files > 0) {
        message += '\n\n音频库已更新！'
      }

      alert(message)

      // 重新加载音频列表
      await loadAudioFiles()
    } catch (error) {
      console.error('扫描失败:', error)
      alert('扫描失败: ' + error)
    } finally {
      setIsScanning(false)
    }
  }

  const loadAudioFiles = async () => {
    try {
      const files = await invoke<AudioFile[]>('get_audio_files')
      setAudioFiles(files)
    } catch (error) {
      console.error('加载失败:', error)
    }
  }

  const handleDelete = () => {
    if (selectedFiles.size === 0) return
    setShowDeleteDialog(true)
  }

  const handleDeleteConfirm = async (deletePhysicalFile: boolean) => {
    setShowDeleteDialog(false)

    try {
      for (const id of selectedFiles) {
        await invoke('delete_audio_file', { id, deletePhysicalFile })
      }
      setSelectedFiles(new Set())
      await loadAudioFiles()
    } catch (error) {
      console.error('删除失败:', error)
      alert('删除失败: ' + error)
    }
  }

  const handleDeleteCancel = () => {
    setShowDeleteDialog(false)
  }

  const handleOpenRecordDialog = () => {
    const now = new Date()
    const defaultFilename = now.toISOString().replace(/[:.]/g, '-').split('T')[0] + '_' +
                           now.toTimeString().split(' ')[0].replace(/:/g, '')
    setRecordingFilename(defaultFilename)
    setShowRecordDialog(true)
  }

  const handleStartRecording = async () => {
    if (!recordingFilename.trim()) {
      alert('请输入文件名')
      return
    }

    try {
      await invoke('start_recording', { filename: recordingFilename })
      setIsRecording(true)
    } catch (error) {
      console.error('开始录音失败:', error)
      alert('开始录音失败: ' + error)
    }
  }

  const handleStopRecording = async () => {
    try {
      await invoke('stop_recording')
      setIsRecording(false)
      setShowRecordDialog(false)
      alert('录音已保存！')
      await loadAudioFiles()
    } catch (error) {
      console.error('停止录音失败:', error)
      alert('停止录音失败: ' + error)
    }
  }

  const handlePlay = async (file: AudioFile) => {
    try {
      // 如果点击的是当前播放的音频
      if (currentAudio && currentAudio.id === file.id) {
        if (isPlaying) {
          await pauseAudio()
        } else {
          await playAudio(file.id, file.original_name)
        }
      } else {
        // 播放新音频
        await playAudio(file.id, file.original_name)
      }
    } catch (error) {
      console.error('播放失败:', error)
    }
  }

  const handleStopCurrent = async () => {
    try {
      await stopAudio()
    } catch (error) {
      console.error('停止失败:', error)
    }
  }

  const formatFileSize = (bytes: number) => {
    if (bytes < 1024) return bytes + ' B'
    if (bytes < 1024 * 1024) return (bytes / 1024).toFixed(1) + ' KB'
    return (bytes / (1024 * 1024)).toFixed(1) + ' MB'
  }

  const formatDuration = (seconds: number) => {
    const mins = Math.floor(seconds / 60)
    const secs = seconds % 60
    return `${mins}:${secs.toString().padStart(2, '0')}`
  }

  const toggleSelection = (id: number) => {
    const newSelection = new Set(selectedFiles)
    if (newSelection.has(id)) {
      newSelection.delete(id)
    } else {
      newSelection.add(id)
    }
    setSelectedFiles(newSelection)
  }

  // 拖放处理 - Tauri 会自动处理文件拖放，不需要手动处理 dragOver/dragLeave/drop
  // 但保留视觉效果的状态管理

  // 页面加载时自动加载音频列表
  useEffect(() => {
    loadAudioFiles()

    // 监听文件拖放事件
    const unlisten = listen<string[]>('tauri://file-drop', async (event) => {
      // 立即隐藏拖放遮罩
      setIsDragging(false)

      const filePaths = event.payload
      const supportedFormats = ['mp3', 'wav', 'ogg', 'flac', 'm4a']

      // 过滤支持的音频文件
      const audioFiles = filePaths.filter(path => {
        const extension = path.split('.').pop()?.toLowerCase()
        return extension && supportedFormats.includes(extension)
      })

      if (audioFiles.length === 0) {
        alert('没有检测到支持的音频文件！\n支持格式：MP3, WAV, OGG, FLAC, M4A')
        return
      }

      // 上传文件
      try {
        for (const filePath of audioFiles) {
          await invoke('upload_audio_file', { filePath })
        }

        alert(`成功上传 ${audioFiles.length} 个文件！`)
        await loadAudioFiles()
      } catch (error) {
        console.error('拖放上传失败:', error)
        alert('拖放上传失败: ' + error)
      }
    })

    // 监听拖放悬停事件
    const unlistenHover = listen('tauri://file-drop-hover', () => {
      setIsDragging(true)
    })

    // 监听拖放取消事件
    const unlistenCancelled = listen('tauri://file-drop-cancelled', () => {
      setIsDragging(false)
    })

    return () => {
      unlisten.then(fn => fn())
      unlistenHover.then(fn => fn())
      unlistenCancelled.then(fn => fn())
    }
  }, [])

  return (
    <div className="h-full flex flex-col bg-white relative">
      {/* 删除确认对话框 */}
      <DeleteConfirmDialog
        isOpen={showDeleteDialog}
        fileCount={selectedFiles.size}
        onConfirm={handleDeleteConfirm}
        onCancel={handleDeleteCancel}
      />

      {/* 拖放遮罩层 */}
      {isDragging && (
        <div className="absolute inset-0 bg-blue-500 bg-opacity-20 border-4 border-dashed border-blue-500 z-50 flex items-center justify-center">
          <div className="bg-white rounded-lg p-8 shadow-xl">
            <Upload size={64} className="mx-auto mb-4 text-blue-600" />
            <p className="text-xl font-bold text-gray-800 mb-2">拖放音频文件到这里</p>
            <p className="text-sm text-gray-600">支持 MP3, WAV, OGG, FLAC, M4A 格式</p>
          </div>
        </div>
      )}

      <div className="border-b border-gray-200 p-6">
        <h2 className="text-2xl font-bold text-gray-800 mb-4">音频库</h2>

        <div className="flex gap-3">
          <button
            onClick={handleUpload}
            className="flex items-center gap-2 px-4 py-2 bg-blue-600 text-white rounded-lg hover:bg-blue-700 transition-colors"
          >
            <Upload size={18} />
            <span>上传音频</span>
          </button>

          <button
            onClick={handleScan}
            disabled={isScanning}
            className="flex items-center gap-2 px-4 py-2 bg-green-600 text-white rounded-lg hover:bg-green-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <RefreshCw size={18} className={isScanning ? 'animate-spin' : ''} />
            <span>{isScanning ? '扫描中...' : '扫描音频'}</span>
          </button>

          <button
            onClick={handleOpenRecordDialog}
            className="flex items-center gap-2 px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors"
          >
            <Mic size={18} />
            <span>录制音频</span>
          </button>

          <button
            onClick={handleDelete}
            disabled={selectedFiles.size === 0}
            className="flex items-center gap-2 px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors disabled:opacity-50 disabled:cursor-not-allowed"
          >
            <Trash2 size={18} />
            <span>删除 {selectedFiles.size > 0 && `(${selectedFiles.size})`}</span>
          </button>

          <div className="flex-1" />

          <div className="relative">
            <Search className="absolute left-3 top-1/2 -translate-y-1/2 text-gray-400" size={18} />
            <input
              type="text"
              placeholder="搜索音频..."
              value={searchTerm}
              onChange={(e) => setSearchTerm(e.target.value)}
              className="pl-10 pr-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-blue-500 w-64"
            />
          </div>
        </div>
      </div>

      <div className="flex-1 overflow-auto p-6">
        {audioFiles.length === 0 ? (
          <div className="flex flex-col items-center justify-center h-full text-gray-400">
            <Music size={64} className="mb-4" />
            <p className="text-lg">还没有音频文件</p>
            <p className="text-sm mt-2">点击"上传音频"、"扫描音频"或拖放文件到此处</p>
            <div className="mt-6 p-6 border-2 border-dashed border-gray-300 rounded-lg">
              <div className="flex items-center gap-3 text-gray-500">
                <RefreshCw size={24} />
                <div>
                  <p className="font-medium">支持自动扫描</p>
                  <p className="text-xs mt-1">扫描音频存储路径中的所有音频文件</p>
                </div>
              </div>
            </div>
            <div className="mt-4 p-6 border-2 border-dashed border-gray-300 rounded-lg">
              <div className="flex items-center gap-3 text-gray-500">
                <Upload size={24} />
                <div>
                  <p className="font-medium">支持拖放上传</p>
                  <p className="text-xs mt-1">MP3, WAV, OGG, FLAC, M4A</p>
                </div>
              </div>
            </div>
          </div>
        ) : (
          <table className="w-full">
            <thead className="border-b border-gray-200">
              <tr className="text-left text-sm text-gray-600">
                <th className="pb-3 w-12">
                  <input
                    type="checkbox"
                    checked={selectedFiles.size === audioFiles.length}
                    onChange={(e) => {
                      if (e.target.checked) {
                        setSelectedFiles(new Set(audioFiles.map(f => f.id)))
                      } else {
                        setSelectedFiles(new Set())
                      }
                    }}
                    className="w-4 h-4"
                  />
                </th>
                <th className="pb-3">文件名</th>
                <th className="pb-3">大小</th>
                <th className="pb-3">时长</th>
                <th className="pb-3">格式</th>
                <th className="pb-3">上传日期</th>
                <th className="pb-3 w-24">操作</th>
              </tr>
            </thead>
            <tbody>
              {audioFiles
                .filter(file =>
                  file.original_name.toLowerCase().includes(searchTerm.toLowerCase())
                )
                .map((file) => (
                  <tr
                    key={file.id}
                    className="border-b border-gray-100 hover:bg-gray-50 transition-colors"
                  >
                    <td className="py-3">
                      <input
                        type="checkbox"
                        checked={selectedFiles.has(file.id)}
                        onChange={() => toggleSelection(file.id)}
                        className="w-4 h-4"
                      />
                    </td>
                    <td className="py-3 text-gray-800">{file.original_name}</td>
                    <td className="py-3 text-gray-600">{formatFileSize(file.file_size)}</td>
                    <td className="py-3 text-gray-600">{formatDuration(file.duration)}</td>
                    <td className="py-3 text-gray-600 uppercase">{file.format}</td>
                    <td className="py-3 text-gray-600">
                      {new Date(file.upload_date).toLocaleDateString('zh-CN')}
                    </td>
                    <td className="py-3">
                      <div className="flex gap-1">
                        {currentAudio && currentAudio.id === file.id && isPlaying ? (
                          <>
                            <button
                              onClick={() => handlePlay(file)}
                              className="p-2 text-orange-600 hover:bg-orange-50 rounded-lg transition-colors"
                              title="暂停"
                            >
                              <Pause size={16} />
                            </button>
                            <button
                              onClick={handleStopCurrent}
                              className="p-2 text-red-600 hover:bg-red-50 rounded-lg transition-colors"
                              title="停止"
                            >
                              <Square size={16} />
                            </button>
                          </>
                        ) : (
                          <button
                            onClick={() => handlePlay(file)}
                            className={`p-2 rounded-lg transition-colors ${
                              currentAudio && currentAudio.id === file.id
                                ? 'text-blue-600 hover:bg-blue-50'
                                : 'text-blue-600 hover:bg-blue-50'
                            }`}
                            title={currentAudio && currentAudio.id === file.id ? '继续播放' : '播放'}
                          >
                            <Play size={16} />
                          </button>
                        )}
                      </div>
                    </td>
                  </tr>
                ))}
            </tbody>
          </table>
        )}
      </div>

      {/* 录制音频对话框 */}
      {showRecordDialog && (
        <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
          <div className="bg-white rounded-lg p-6 w-96">
            <h3 className="text-xl font-bold mb-4">录制音频</h3>
            <div className="mb-4">
              <label className="block text-sm font-medium text-gray-700 mb-2">
                文件名
              </label>
              <input
                type="text"
                value={recordingFilename}
                onChange={(e) => setRecordingFilename(e.target.value)}
                placeholder="请输入文件名"
                className="w-full px-4 py-2 border border-gray-300 rounded-lg focus:outline-none focus:ring-2 focus:ring-purple-500"
                disabled={isRecording}
              />
              <p className="text-xs text-gray-500 mt-1">文件将保存为 WAV 格式</p>
            </div>
            {isRecording && (
              <div className="mb-4 p-3 bg-red-50 border border-red-200 rounded-lg">
                <div className="flex items-center gap-2 text-red-600">
                  <div className="w-3 h-3 bg-red-600 rounded-full animate-pulse" />
                  <span className="font-medium">正在录音中...</span>
                </div>
              </div>
            )}
            <div className="flex justify-end gap-2">
              {!isRecording ? (
                <>
                  <button
                    onClick={() => setShowRecordDialog(false)}
                    className="px-4 py-2 text-gray-600 hover:bg-gray-100 rounded-lg transition-colors"
                  >
                    取消
                  </button>
                  <button
                    onClick={handleStartRecording}
                    className="flex items-center gap-2 px-4 py-2 bg-purple-600 text-white rounded-lg hover:bg-purple-700 transition-colors"
                  >
                    <Mic size={16} />
                    <span>开始录音</span>
                  </button>
                </>
              ) : (
                <button
                  onClick={handleStopRecording}
                  className="flex items-center gap-2 px-4 py-2 bg-red-600 text-white rounded-lg hover:bg-red-700 transition-colors"
                >
                  <Square size={16} />
                  <span>结束录音</span>
                </button>
              )}
            </div>
          </div>
        </div>
      )}
    </div>
  )
}
