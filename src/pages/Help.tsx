import { useState } from 'react'
import { Book, Clock, List, Music, Settings, Play, Upload, Trash2, HelpCircle, ChevronRight } from 'lucide-react'

export default function Help() {
  const [activeSection, setActiveSection] = useState('start')

  const sections = [
    { id: 'start', title: '快速开始', icon: Play },
    { id: 'features', title: '功能介绍', icon: Book },
    { id: 'troubleshooting', title: '常见问题', icon: HelpCircle },
    { id: 'about', title: '关于应用', icon: Settings },
  ]

  const renderContent = () => {
    switch (activeSection) {
      case 'start':
        return (
          <div className="space-y-6">
            <div className="bg-blue-50 border border-blue-200 rounded-lg p-6">
              <h3 className="text-xl font-bold text-blue-800 mb-3">快速开始指南</h3>
              <p className="text-blue-700 mb-4">欢迎使用磨耳朵！按照以下步骤，5分钟即可开始您的英语听力训练之旅：</p>
            </div>

            <div className="grid gap-4">
              <div className="flex gap-4">
                <div className="flex-shrink-0 w-12 h-12 bg-green-100 rounded-full flex items-center justify-center">
                  <span className="text-green-600 font-bold">1</span>
                </div>
                <div className="flex-1">
                  <h4 className="font-semibold text-gray-800">上传音频文件</h4>
                  <p className="text-gray-600">点击"上传音频"按钮，选择MP3、WAV、OGG、FLAC或M4A格式的英语听力材料。支持批量上传和拖拽上传。</p>
                  <p className="text-sm text-gray-500 mt-1">提示：建议选择语速适中、发音清晰的英语音频。</p>
                </div>
              </div>

              <div className="flex gap-4">
                <div className="flex-shrink-0 w-12 h-12 bg-blue-100 rounded-full flex items-center justify-center">
                  <span className="text-blue-600 font-bold">2</span>
                </div>
                <div className="flex-1">
                  <h4 className="font-semibold text-gray-800">创建播放列表</h4>
                  <p className="text-gray-600">在播放列表页面创建不同主题的列表，如"早安英语"、"睡前故事"、"单词记忆"等。每个列表可以包含多个音频文件。</p>
                </div>
              </div>

              <div className="flex gap-4">
                <div className="flex-shrink-0 w-12 h-12 bg-purple-100 rounded-full flex items-center justify-center">
                  <span className="text-purple-600 font-bold">3</span>
                </div>
                <div className="flex-1">
                  <h4 className="font-semibold text-gray-800">设置定时任务</h4>
                  <p className="text-gray-600">在定时任务页面创建早晨播放任务，设置适合的播放时间（建议6:30-7:30）。可选择重复模式和工作日/周末。</p>
                </div>
              </div>

              <div className="flex gap-4">
                <div className="flex-shrink-0 w-12 h-12 bg-orange-100 rounded-full flex items-center justify-center">
                  <span className="text-orange-600 font-bold">4</span>
                </div>
                <div className="flex-1">
                  <h4 className="font-semibold text-gray-800">享受学习时光</h4>
                  <p className="text-gray-600">系统会在设定时间自动播放，帮助孩子在起床时自然地接触英语，培养语感和学习习惯。</p>
                </div>
              </div>
            </div>
          </div>
        )

      case 'features':
        return (
          <div className="space-y-6">
            <div className="grid grid-cols-1 md:grid-cols-2 gap-6">
              <div className="bg-white border border-gray-200 rounded-lg p-6">
                <div className="flex items-center gap-3 mb-4">
                  <Music className="text-blue-600" size={24} />
                  <h3 className="text-lg font-semibold text-gray-800">音频管理</h3>
                </div>
                <ul className="space-y-2 text-sm text-gray-600">
                  <li>• 支持5种音频格式</li>
                  <li>• 批量上传和管理</li>
                  <li>• 拖拽上传功能</li>
                  <li>• 文件搜索和过滤</li>
                  <li>• 在线测试播放</li>
                </ul>
              </div>

              <div className="bg-white border border-gray-200 rounded-lg p-6">
                <div className="flex items-center gap-3 mb-4">
                  <List className="text-green-600" size={24} />
                  <h3 className="text-lg font-semibold text-gray-800">播放列表</h3>
                </div>
                <ul className="space-y-2 text-sm text-gray-600">
                  <li>• 创建多个播放列表</li>
                  <li>• 四种播放模式</li>
                  <li>• 音频排序功能</li>
                  <li>• 列表复制和编辑</li>
                  <li>• 播放状态管理</li>
                </ul>
              </div>

              <div className="bg-white border border-gray-200 rounded-lg p-6">
                <div className="flex items-center gap-3 mb-4">
                  <Clock className="text-purple-600" size={24} />
                  <h3 className="text-lg font-semibold text-gray-800">定时任务</h3>
                </div>
                <ul className="space-y-2 text-sm text-gray-600">
                  <li>• 灵活的时间设置</li>
                  <li>• 多种重复模式</li>
                  <li>• 自定义星期选择</li>
                  <li>• 音量和渐强控制</li>
                  <li>• 任务优先级管理</li>
                </ul>
              </div>

              <div className="bg-white border border-gray-200 rounded-lg p-6">
                <div className="flex items-center gap-3 mb-4">
                  <Play className="text-orange-600" size={24} />
                  <h3 className="text-lg font-semibold text-gray-800">播放控制</h3>
                </div>
                <ul className="space-y-2 text-sm text-gray-600">
                  <li>• 全局播放控制器</li>
                  <li>• 播放/暂停/停止</li>
                  <li>• 音量调节</li>
                  <li>• 播放进度显示</li>
                  <li>• 实时状态反馈</li>
                </ul>
              </div>
            </div>
          </div>
        )

      case 'troubleshooting':
        return (
          <div className="space-y-6">
            <div className="bg-yellow-50 border border-yellow-200 rounded-lg p-6">
              <h3 className="text-lg font-semibold text-yellow-800 mb-3">常见问题解答</h3>
              <p className="text-yellow-700">这里是一些常见问题的解决方案，如果遇到其他问题，请参考下方的联系信息。</p>
            </div>

            <div className="space-y-4">
              <div className="bg-white border border-gray-200 rounded-lg p-4">
                <h4 className="font-semibold text-gray-800 mb-2">Q: 为什么音频播放失败？</h4>
                <p className="text-gray-600">A: 请检查：</p>
                <ul className="list-disc list-inside space-y-1 text-sm text-gray-600 ml-4">
                  <li>音频文件格式是否支持（MP3、WAV、OGG、FLAC、M4A）</li>
                  <li>音频文件是否损坏</li>
                  <li>系统音量是否开启</li>
                  <li>应用是否有音频播放权限</li>
                </ul>
              </div>

              <div className="bg-white border border-gray-200 rounded-lg p-4">
                <h4 className="font-semibold text-gray-800 mb-2">Q: 定时任务没有执行怎么办？</h4>
                <p className="text-gray-600">A: 请检查：</p>
                <ul className="list-disc list-inside space-y-1 text-sm text-gray-600 ml-4">
                  <li>任务是否已启用（蓝色高亮）</li>
                  <li>播放列表是否包含音频</li>
                  <li>播放列表是否关联到任务</li>
                  <li>电脑时间是否准确</li>
                  <li>应用是否正在运行</li>
                </ul>
              </div>

              <div className="bg-white border border-gray-200 rounded-lg p-4">
                <h4 className="font-semibold text-gray-800 mb-2">Q: 如何备份和恢复数据？</h4>
                <p className="text-gray-600">A: 在设置页面的"数据管理"部分，可以导出配置和数据到JSON文件，也可以从文件导入数据。</p>
              </div>

              <div className="bg-white border border-gray-200 rounded-lg p-4">
                <h4 className="font-semibold text-gray-800 mb-2">Q: 拖拽上传不工作？</h4>
<p className="text-gray-600">A: 由于浏览器安全限制，拖拽功能可能不会立即生效。建议使用"上传音频"按钮，或者在拖拽后按照提示操作。</p>
              </div>
            </div>
          </div>
        )

      case 'about':
        return (
          <div className="space-y-6">
            <div className="bg-white border border-gray-200 rounded-lg p-6">
              <h3 className="text-xl font-bold text-gray-800 mb-4">关于磨耳朵</h3>
              <div className="space-y-4">
                <div>
                  <h4 className="font-semibold text-gray-700">应用信息</h4>
                  <div className="bg-gray-50 rounded-lg p-4 mt-2">
                    <table className="w-full text-sm">
                      <tr>
                        <td className="font-medium text-gray-600 pr-4">版本：</td>
                        <td>0.1.0</td>
                      </tr>
                      <tr>
                        <td className="font-medium text-gray-600 pr-4">发布日期：</td>
                        <td>2025年10月14日</td>
                      </tr>
                      <tr>
                        <td className="font-medium text-gray-600 pr-4">开发框架：</td>
                        <td>Tauri + React + Rust</td>
                      </tr>
                      <tr>
                        <td className="font-medium text-gray-600 pr-4">支持平台：</td>
                        <td>Windows, macOS, Linux</td>
                      </tr>
                    </table>
                  </div>
                </div>

                <div>
                  <h4 className="font-semibold text-gray-700">适用对象</h4>
                  <ul className="list-disc list-inside text-sm text-gray-600 ml-4 space-y-1">
                    <li>小学生（6-12岁）</li>
                    <li>初中生（13-15岁）</li>
                    <li>高中生（16-18岁）</li>
                    <li>英语学习者</li>
                    <li>关注孩子教育的家长</li>
                  </ul>
                </div>

                <div>
                  <h4 className="font-semibold text-gray-700">主要功能</h4>
                  <p className="text-sm text-gray-600 mt-2">
                    磨耳朵通过定时播放英语音频，帮助学生在日常生活中自然接触英语，
                    培养语感、提高听力水平、养成良好学习习惯。
                  </p>
                </div>

                <div>
                  <h4 className="font-semibold text-gray-700">教育理念</h4>
                  <p className="text-sm text-gray-600 mt-2">
                    通过"浸入式"学习方法，让学生在无压力的环境中接触英语，
                    借助建立英语思维模式，提高语言习得效率。
                  </p>
                </div>

                <div>
                  <h4 className="font-semibold text-gray-700">版权信息</h4>
                  <p className="text-sm text-gray-600 mt-2">
                    本软件遵循开源协议发布，源代码完全开放。
                    用户可自由使用、修改和分发，但请保留原作者信息。
                  </p>
                </div>
              </div>
            </div>
          </div>
        )

      default:
        return null
    }
  }

  return (
    <div className="h-full flex bg-gray-50">
      <div className="w-64 border-r border-gray-200 bg-white">
        <div className="p-4 border-b border-gray-200">
          <h2 className="text-xl font-bold text-gray-800">帮助中心</h2>
        </div>
        <nav className="p-4">
          {sections.map((section) => (
            <button
              key={section.id}
              onClick={() => setActiveSection(section.id)}
              className={`w-full flex items-center gap-3 px-4 py-3 rounded-lg mb-2 transition-colors text-left ${
                activeSection === section.id
                  ? 'bg-blue-50 text-blue-600 border border-blue-200'
                  : 'text-gray-700 hover:bg-gray-50'
              }`}
            >
              <section.icon size={20} />
              <span>{section.title}</span>
              <ChevronRight size={16} className="ml-auto" />
            </button>
          ))}
        </nav>
      </div>

      <div className="flex-1 overflow-auto p-6">
        <div className="max-w-4xl mx-auto">
          {renderContent()}
        </div>
      </div>
    </div>
  )
}
