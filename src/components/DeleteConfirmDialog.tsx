import { useState } from 'react'
import { AlertTriangle, X } from 'lucide-react'

interface DeleteConfirmDialogProps {
  isOpen: boolean
  fileCount: number
  onConfirm: (deletePhysicalFile: boolean) => void
  onCancel: () => void
}

export default function DeleteConfirmDialog({
  isOpen,
  fileCount,
  onConfirm,
  onCancel,
}: DeleteConfirmDialogProps) {
  const [deletePhysicalFile, setDeletePhysicalFile] = useState(false)

  if (!isOpen) return null

  const handleConfirm = () => {
    onConfirm(deletePhysicalFile)
    setDeletePhysicalFile(false) // 重置状态
  }

  const handleCancel = () => {
    setDeletePhysicalFile(false) // 重置状态
    onCancel()
  }

  return (
    <div className="fixed inset-0 bg-black bg-opacity-50 flex items-center justify-center z-50">
      <div className="bg-white rounded-lg shadow-xl max-w-md w-full mx-4">
        {/* 头部 */}
        <div className="flex items-center justify-between p-6 border-b border-gray-200">
          <div className="flex items-center gap-3">
            <div className="w-10 h-10 bg-red-100 rounded-full flex items-center justify-center">
              <AlertTriangle className="text-red-600" size={20} />
            </div>
            <h3 className="text-lg font-semibold text-gray-800">确认删除</h3>
          </div>
          <button
            onClick={handleCancel}
            className="text-gray-400 hover:text-gray-600 transition-colors"
          >
            <X size={20} />
          </button>
        </div>

        {/* 内容 */}
        <div className="p-6">
          <p className="text-gray-700 mb-4">
            确定要删除选中的 <span className="font-semibold text-red-600">{fileCount}</span> 个文件吗？
          </p>

          {/* 勾选框 */}
          <label className="flex items-start gap-3 p-4 bg-gray-50 rounded-lg border border-gray-200 cursor-pointer hover:bg-gray-100 transition-colors">
            <input
              type="checkbox"
              checked={deletePhysicalFile}
              onChange={(e) => setDeletePhysicalFile(e.target.checked)}
              className="mt-1 w-4 h-4 text-red-600 rounded focus:ring-2 focus:ring-red-500"
            />
            <div className="flex-1">
              <p className="font-medium text-gray-800">同时删除硬盘上的文件</p>
              <p className="text-sm text-gray-600 mt-1">
                {deletePhysicalFile ? (
                  <span className="text-red-600">⚠️ 警告：文件将被永久删除，无法恢复！</span>
                ) : (
                  <span>仅从音频库中移除，保留硬盘文件</span>
                )}
              </p>
            </div>
          </label>
        </div>

        {/* 底部按钮 */}
        <div className="flex gap-3 p-6 border-t border-gray-200">
          <button
            onClick={handleCancel}
            className="flex-1 px-4 py-2 border border-gray-300 text-gray-700 rounded-lg hover:bg-gray-50 transition-colors"
          >
            取消
          </button>
          <button
            onClick={handleConfirm}
            className={`flex-1 px-4 py-2 rounded-lg transition-colors ${
              deletePhysicalFile
                ? 'bg-red-600 text-white hover:bg-red-700'
                : 'bg-orange-600 text-white hover:bg-orange-700'
            }`}
          >
            {deletePhysicalFile ? '永久删除' : '仅移除'}
          </button>
        </div>
      </div>
    </div>
  )
}
