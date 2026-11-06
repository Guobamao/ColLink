<template>
  <div class="h-screen w-screen flex flex-col bg-surface text-primary-text select-none overflow-hidden font-sans">
    <!-- Top Header Bar with Native Drag Region & Window Controls -->
    <div
      data-tauri-drag-region
      class="h-10 pl-3 pr-0 bg-surface border-b border-border flex items-center justify-between z-50 text-xs shrink-0 shadow-2xs"
    >
      <!-- Left: Back Button & Title -->
      <div class="flex items-center gap-2">
        <button
          type="button"
          class="flex items-center gap-1.5 px-2.5 py-1 rounded-md text-xs font-medium text-muted hover:text-primary-text hover:bg-surface-hover border border-border transition-colors cursor-pointer"
          title="返回工作区画布 (Esc / Ctrl+,)"
          @click="$emit('back')"
        >
          <ChevronLeft class="w-3.5 h-3.5" />
          <span>返回画板</span>
        </button>

        <div class="w-px h-3.5 bg-border mx-1"></div>

        <div class="flex items-center gap-1.5 text-xs text-primary-text font-semibold">
          <Settings class="w-3.5 h-3.5 text-primary" />
          <span>系统设置中心</span>
          <span class="text-[10px] text-muted font-normal ml-1">Preferences</span>
        </div>
      </div>

      <!-- Right: Window Controls (Tauri Custom Titlebar) -->
      <div v-if="isTauri" class="flex items-center h-10 select-none">
        <button
          type="button"
          class="w-10 h-10 flex items-center justify-center text-muted hover:text-primary-text hover:bg-surface-hover transition-colors cursor-pointer"
          title="最小化"
          @click="handleMinimize"
        >
          <Minus class="w-3.5 h-3.5" />
        </button>

        <button
          type="button"
          class="w-10 h-10 flex items-center justify-center text-muted hover:text-primary-text hover:bg-surface-hover transition-colors cursor-pointer"
          :title="isMaximized ? '向下还原' : '最大化'"
          @click="handleToggleMaximize"
        >
          <Copy v-if="isMaximized" class="w-3 h-3 rotate-180" />
          <Square v-else class="w-3 h-3" />
        </button>

        <button
          type="button"
          class="w-10 h-10 flex items-center justify-center text-muted hover:text-white hover:bg-red-600 transition-colors cursor-pointer"
          title="隐藏到托盘 / 关闭"
          @click="handleClose"
        >
          <X class="w-3.5 h-3.5" />
        </button>
      </div>
    </div>

    <!-- Main Content Area: Split Sidebar & Settings Panel -->
    <div class="flex-1 flex min-h-0 overflow-hidden">
      <!-- Left Sidebar Navigation -->
      <div class="w-56 border-r border-border bg-surface-subtle/40 p-3 flex flex-col gap-1 shrink-0 overflow-y-auto">
        <div class="px-2 py-1.5 text-[10px] font-semibold text-muted tracking-wider uppercase">
          设置分类
        </div>

        <button
          v-for="item in navItems"
          :key="item.id"
          type="button"
          class="flex items-center gap-2.5 px-3 py-2 rounded-lg text-xs font-medium transition-all duration-150 cursor-pointer text-left"
          :class="[
            activeTab === item.id
              ? 'bg-primary-light/50 text-primary font-semibold shadow-2xs'
              : 'text-muted hover:text-primary-text hover:bg-surface-hover/80'
          ]"
          @click="activeTab = item.id"
        >
          <component :is="item.icon" class="w-4 h-4 shrink-0" :class="activeTab === item.id ? 'text-primary' : 'text-muted'" />
          <div class="flex flex-col min-w-0">
            <span class="leading-tight">{{ item.label }}</span>
            <span class="text-[10px] text-muted truncate mt-0.5">{{ item.subLabel }}</span>
          </div>
        </button>

        <div class="mt-auto pt-3 border-t border-border">
          <div class="px-2 py-1 text-[11px] text-muted flex items-center justify-between">
            <span>快捷键返回</span>
            <kbd class="px-1.5 py-0.5 rounded bg-surface border border-border text-[10px] font-mono">Esc</kbd>
          </div>
        </div>
      </div>

      <!-- Right Settings Content Panel -->
      <div class="flex-1 overflow-y-auto p-8 bg-surface-subtle/20 custom-scrollbar">
        <div class="max-w-3xl mx-auto space-y-6">

          <!-- TAB 1: 外观与主题 (Appearance & Theme) -->
          <div v-if="activeTab === 'appearance'" class="space-y-6">
            <div>
              <h2 class="text-base font-bold text-primary-text">外观与主题</h2>
              <p class="text-xs text-muted mt-0.5">定制个性化视觉风格、深浅模式及全站品牌主题色</p>
            </div>

            <!-- 1.1 主题模式 -->
            <div class="p-4 rounded-xl border border-border bg-surface shadow-2xs space-y-3">
              <div class="flex items-center justify-between">
                <div>
                  <h3 class="text-xs font-semibold text-primary-text">主题模式</h3>
                  <p class="text-[11px] text-muted mt-0.5">选择偏好的显示模式，切换时将伴随平滑波纹过渡动画</p>
                </div>
              </div>

              <div class="grid grid-cols-3 gap-3 pt-1">
                <button
                  type="button"
                  class="flex flex-col items-center justify-center gap-2 p-3.5 rounded-lg border text-xs font-medium transition-all cursor-pointer"
                  :class="[
                    themeStore.mode === 'light'
                      ? 'border-primary bg-primary-light/30 text-primary ring-1 ring-primary/40'
                      : 'border-border bg-surface hover:bg-surface-hover text-muted hover:text-primary-text'
                  ]"
                  @click="themeStore.setMode('light', $event)"
                >
                  <Sun class="w-5 h-5 text-amber-500" />
                  <span>浅色模式</span>
                </button>

                <button
                  type="button"
                  class="flex flex-col items-center justify-center gap-2 p-3.5 rounded-lg border text-xs font-medium transition-all cursor-pointer"
                  :class="[
                    themeStore.mode === 'dark'
                      ? 'border-primary bg-primary-light/30 text-primary ring-1 ring-primary/40'
                      : 'border-border bg-surface hover:bg-surface-hover text-muted hover:text-primary-text'
                  ]"
                  @click="themeStore.setMode('dark', $event)"
                >
                  <Moon class="w-5 h-5 text-indigo-400" />
                  <span>深色模式</span>
                </button>

                <button
                  type="button"
                  class="flex flex-col items-center justify-center gap-2 p-3.5 rounded-lg border text-xs font-medium transition-all cursor-pointer"
                  :class="[
                    themeStore.mode === 'auto'
                      ? 'border-primary bg-primary-light/30 text-primary ring-1 ring-primary/40'
                      : 'border-border bg-surface hover:bg-surface-hover text-muted hover:text-primary-text'
                  ]"
                  @click="themeStore.setMode('auto', $event)"
                >
                  <Monitor class="w-5 h-5 text-blue-500" />
                  <span>跟随系统</span>
                </button>
              </div>
            </div>

            <!-- 1.2 全站品牌强调色 -->
            <div class="p-4 rounded-xl border border-border bg-surface shadow-2xs space-y-3">
              <div class="flex items-center justify-between">
                <div>
                  <h3 class="text-xs font-semibold text-primary-text">全站品牌强调色</h3>
                  <p class="text-[11px] text-muted mt-0.5">实时应用至按钮高亮、连线脉冲、边框与交互视觉焦点</p>
                </div>
                <!-- Preview Tag -->
                <div class="flex items-center gap-1.5 px-2 py-0.5 rounded border border-primary bg-primary-light/50 text-primary text-[11px] font-medium">
                  <span class="w-2 h-2 rounded-full bg-primary animate-pulse"></span>
                  <span>效果预览</span>
                </div>
              </div>

              <div class="grid grid-cols-4 gap-2.5 pt-1">
                <button
                  v-for="color in ACCENT_COLORS"
                  :key="color.id"
                  type="button"
                  class="flex items-center gap-2.5 p-2 rounded-lg border text-xs transition-all cursor-pointer text-left group"
                  :class="[
                    themeStore.accentColorId === color.id
                      ? 'border-primary bg-primary-light/20 ring-1 ring-primary/30 font-medium'
                      : 'border-border bg-surface hover:bg-surface-hover'
                  ]"
                  @click="themeStore.setAccentColor(color.id)"
                >
                  <span
                    class="w-4 h-4 rounded-full shrink-0 shadow-xs flex items-center justify-center text-white text-[9px]"
                    :style="{ backgroundColor: color.hex }"
                  >
                    <Check v-if="themeStore.accentColorId === color.id" class="w-3 h-3 stroke-[3]" />
                  </span>
                  <div class="flex flex-col min-w-0">
                    <span class="text-xs text-primary-text truncate leading-tight">{{ color.name }}</span>
                    <span class="text-[10px] text-muted font-mono leading-tight">{{ color.hex }}</span>
                  </div>
                </button>
              </div>
            </div>

            <!-- 1.3 画布底纹样式 -->
            <div class="p-4 rounded-xl border border-border bg-surface shadow-2xs space-y-3">
              <div>
                <h3 class="text-xs font-semibold text-primary-text">画布底纹网格</h3>
                <p class="text-[11px] text-muted mt-0.5">选择主画板背景纹理的排布风格</p>
              </div>

              <div class="grid grid-cols-2 gap-3 pt-1">
                <button
                  type="button"
                  class="flex items-center gap-3 p-3 rounded-lg border text-xs transition-all cursor-pointer"
                  :class="[
                    themeStore.gridPattern === 'dots'
                      ? 'border-primary bg-primary-light/20 text-primary font-medium ring-1 ring-primary/30'
                      : 'border-border bg-surface hover:bg-surface-hover text-muted hover:text-primary-text'
                  ]"
                  @click="themeStore.setGridPattern('dots')"
                >
                  <div class="w-8 h-8 rounded border border-border bg-surface-subtle flex items-center justify-center shrink-0">
                    <div class="grid grid-cols-2 gap-1">
                      <span class="w-1 h-1 rounded-full bg-muted"></span>
                      <span class="w-1 h-1 rounded-full bg-muted"></span>
                      <span class="w-1 h-1 rounded-full bg-muted"></span>
                      <span class="w-1 h-1 rounded-full bg-muted"></span>
                    </div>
                  </div>
                  <div class="flex flex-col text-left">
                    <span class="text-xs text-primary-text font-medium">点阵网格 (Dots)</span>
                    <span class="text-[10px] text-muted">简洁通透，聚焦血缘卡片</span>
                  </div>
                </button>

                <button
                  type="button"
                  class="flex items-center gap-3 p-3 rounded-lg border text-xs transition-all cursor-pointer"
                  :class="[
                    themeStore.gridPattern === 'lines'
                      ? 'border-primary bg-primary-light/20 text-primary font-medium ring-1 ring-primary/30'
                      : 'border-border bg-surface hover:bg-surface-hover text-muted hover:text-primary-text'
                  ]"
                  @click="themeStore.setGridPattern('lines')"
                >
                  <div class="w-8 h-8 rounded border border-border bg-surface-subtle flex items-center justify-center shrink-0">
                    <div class="w-4 h-4 border border-dashed border-muted"></div>
                  </div>
                  <div class="flex flex-col text-left">
                    <span class="text-xs text-primary-text font-medium">方格线条 (Lines)</span>
                    <span class="text-[10px] text-muted">工程蓝图质感，便于坐标对齐</span>
                  </div>
                </button>
              </div>
            </div>
          </div>

          <!-- TAB 2: MCP 协同与 AI (Model Context Protocol) -->
          <div v-if="activeTab === 'mcp'" class="space-y-6">
            <div>
              <h2 class="text-base font-bold text-primary-text">MCP 协同与 AI</h2>
              <p class="text-xs text-muted mt-0.5">通过 Model Context Protocol 协议无缝接入 Claude Desktop、Cursor 或 Antigravity 等 AI 编程助手</p>
            </div>

            <!-- 2.1 服务核心路径与状态 -->
            <div class="p-4 rounded-xl border border-border bg-surface shadow-2xs space-y-3">
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2">
                  <div class="w-6 h-6 rounded-md bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 flex items-center justify-center shrink-0 border border-emerald-500/20">
                    <Cpu class="w-3.5 h-3.5" />
                  </div>
                  <div>
                    <h3 class="text-xs font-semibold text-primary-text">ColLink MCP 服务状态</h3>
                    <p class="text-[11px] text-muted">基于 JSON-RPC 2.0 与标准输入输出 (stdio) 进行本地进程间双向通信</p>
                  </div>
                </div>
                <span class="inline-flex items-center gap-1 px-2 py-0.5 rounded-full text-[10px] font-medium bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/20">
                  <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
                  服务就绪 (Ready)
                </span>
              </div>

              <!-- Executable Path Box -->
              <div class="flex items-center gap-2 p-2 rounded-lg bg-surface-subtle border border-border">
                <div class="flex flex-col min-w-0 flex-1">
                  <span class="text-[10px] text-muted font-medium">MCP 服务二进制路径 (collink-mcp.exe):</span>
                  <span class="text-xs font-mono text-primary-text truncate select-all" :title="appInfo?.mcp_path">
                    {{ appInfo?.mcp_path || '正在读取可执行路径...' }}
                  </span>
                </div>
                <button
                  type="button"
                  class="px-2.5 py-1 text-xs rounded border border-border bg-surface hover:bg-surface-hover text-muted hover:text-primary-text transition-colors shrink-0 cursor-pointer flex items-center gap-1"
                  @click="copyText(appInfo?.mcp_path || '', 'MCP 二进制路径已复制')"
                >
                  <Copy class="w-3 h-3" />
                  <span>复制路径</span>
                </button>
              </div>
            </div>

            <!-- 2.2 客户端接入配置一键生成器 -->
            <div class="p-4 rounded-xl border border-border bg-surface shadow-2xs space-y-3">
              <div class="flex items-center justify-between">
                <div>
                  <h3 class="text-xs font-semibold text-primary-text">客户端一键接入配置</h3>
                  <p class="text-[11px] text-muted mt-0.5">将生成的 JSON 配置片段直接粘贴至您的 AI 客户端配置文件中</p>
                </div>
                <!-- Client Switcher Tabs -->
                <div class="flex items-center p-0.5 rounded-lg bg-surface-subtle border border-border text-xs">
                  <button
                    v-for="client in mcpClients"
                    :key="client.id"
                    type="button"
                    class="px-2.5 py-1 rounded-md transition-all cursor-pointer text-xs"
                    :class="[
                      activeMcpClient === client.id
                        ? 'bg-surface text-primary font-semibold shadow-2xs'
                        : 'text-muted hover:text-primary-text'
                    ]"
                    @click="activeMcpClient = client.id"
                  >
                    {{ client.name }}
                  </button>
                </div>
              </div>

              <!-- Config Code Snippet Box -->
              <div class="relative rounded-lg border border-border bg-canvas p-3 font-mono text-xs overflow-x-auto">
                <pre class="text-primary-text text-[11px] leading-relaxed select-all"><code>{{ activeClientConfigJson }}</code></pre>
                <div class="absolute top-2.5 right-2.5 flex items-center gap-1.5">
                  <button
                    type="button"
                    class="px-2.5 py-1 rounded border border-border bg-surface hover:bg-surface-hover text-primary-text text-[11px] transition-colors cursor-pointer flex items-center gap-1 shadow-2xs"
                    @click="copyText(activeClientConfigJson, 'JSON 配置代码已复制到剪贴板')"
                  >
                    <Copy class="w-3 h-3 text-muted" />
                    <span>复制配置</span>
                  </button>
                </div>
              </div>
            </div>

            <!-- 2.3 内建 8 大 MCP 工具能力浏览器 -->
            <div class="p-4 rounded-xl border border-border bg-surface shadow-2xs space-y-3">
              <div>
                <h3 class="text-xs font-semibold text-primary-text">内建工具集 (8 个 Tools)</h3>
                <p class="text-[11px] text-muted mt-0.5">AI 助手接入后可直接调用的自动化分析工具清单</p>
              </div>

              <div class="grid grid-cols-2 gap-2.5">
                <div
                  v-for="tool in mcpToolList"
                  :key="tool.name"
                  class="p-2.5 rounded-lg border border-border bg-surface-subtle/30 space-y-1 text-left"
                >
                  <div class="flex items-center justify-between">
                    <span class="text-xs font-mono font-semibold text-primary">{{ tool.name }}</span>
                    <span class="text-[9px] px-1 py-0.2 rounded bg-surface border border-border text-muted font-mono">{{ tool.category }}</span>
                  </div>
                  <p class="text-[11px] text-muted leading-tight">{{ tool.description }}</p>
                </div>
              </div>
            </div>
          </div>

          <!-- TAB 3: 常规与偏好 (General & Canvas) -->
          <div v-if="activeTab === 'general'" class="space-y-6">
            <div>
              <h2 class="text-base font-bold text-primary-text">常规与画布偏好</h2>
              <p class="text-xs text-muted mt-0.5">管理窗口运行、生命周期与画板核心交互行为</p>
            </div>

            <!-- 3.1 窗口关闭行为 -->
            <div class="p-4 rounded-xl border border-border bg-surface shadow-2xs space-y-3">
              <div>
                <h3 class="text-xs font-semibold text-primary-text">窗口关闭与后台驻留</h3>
                <p class="text-[11px] text-muted mt-0.5">决定点击右上角关闭按钮或按下 Alt+F4 时的响应动作</p>
              </div>

              <div class="space-y-2">
                <label
                  class="flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors"
                  :class="[
                    closeToTray
                      ? 'border-primary bg-primary-light/20 text-primary-text'
                      : 'border-border bg-surface hover:bg-surface-hover text-muted'
                  ]"
                >
                  <input
                    type="radio"
                    name="closeBehavior"
                    :checked="closeToTray"
                    class="mt-0.5 text-primary focus:ring-primary"
                    @change="setCloseToTray(true)"
                  />
                  <div class="flex flex-col">
                    <span class="text-xs font-medium text-primary-text">隐藏至系统托盘继续在后台驻留（推荐）</span>
                    <span class="text-[11px] text-muted mt-0.5">关闭窗口时不中断进程，双击托盘图标即可极速唤起主界面</span>
                  </div>
                </label>

                <label
                  class="flex items-start gap-3 p-3 rounded-lg border cursor-pointer transition-colors"
                  :class="[
                    !closeToTray
                      ? 'border-primary bg-primary-light/20 text-primary-text'
                      : 'border-border bg-surface hover:bg-surface-hover text-muted'
                  ]"
                >
                  <input
                    type="radio"
                    name="closeBehavior"
                    :checked="!closeToTray"
                    class="mt-0.5 text-primary focus:ring-primary"
                    @change="setCloseToTray(false)"
                  />
                  <div class="flex flex-col">
                    <span class="text-xs font-medium text-primary-text">直接彻底退出应用程序</span>
                    <span class="text-[11px] text-muted mt-0.5">关闭主窗口时直接销毁进程并释放全部系统内存资源</span>
                  </div>
                </label>
              </div>
            </div>

            <!-- 3.2 画布左下角操作提示 -->
            <div class="p-4 rounded-xl border border-border bg-surface shadow-2xs flex items-center justify-between">
              <div>
                <h3 class="text-xs font-semibold text-primary-text">画布左下角操作提示胶囊</h3>
                <p class="text-[11px] text-muted mt-0.5">在主画板左下角常驻展示字段血缘聚焦、连线与脉冲快捷操作指南</p>
              </div>

              <button
                type="button"
                class="w-10 h-5 rounded-full transition-colors relative cursor-pointer focus:outline-none"
                :class="showFloatingTip ? 'bg-primary' : 'bg-surface-hover border border-border'"
                @click="toggleFloatingTip"
              >
                <span
                  class="w-3.5 h-3.5 rounded-full bg-white shadow-xs absolute top-0.75 transition-transform"
                  :class="showFloatingTip ? 'right-0.75' : 'left-0.75'"
                ></span>
              </button>
            </div>

            <!-- 3.3 默认连线算法 -->
            <div class="p-4 rounded-xl border border-border bg-surface shadow-2xs space-y-3">
              <div>
                <h3 class="text-xs font-semibold text-primary-text">默认连线路由算法</h3>
                <p class="text-[11px] text-muted mt-0.5">定义画布中字段级映射连线的路径计算策略</p>
              </div>

              <div class="grid grid-cols-2 gap-3 pt-1">
                <button
                  type="button"
                  class="flex items-start gap-3 p-3 rounded-lg border text-left transition-all cursor-pointer"
                  :class="[
                    defaultEdgeType === 'smartObstacle'
                      ? 'border-primary bg-primary-light/20 ring-1 ring-primary/30'
                      : 'border-border bg-surface hover:bg-surface-hover'
                  ]"
                  @click="setDefaultEdgeType('smartObstacle')"
                >
                  <Route class="w-4 h-4 text-primary shrink-0 mt-0.5" />
                  <div class="flex flex-col">
                    <span class="text-xs font-medium text-primary-text">智能避障折线 (Smart Obstacle)</span>
                    <span class="text-[10px] text-muted mt-0.5">自动绕行中间数据表卡片，画面整洁不穿模</span>
                  </div>
                </button>

                <button
                  type="button"
                  class="flex items-start gap-3 p-3 rounded-lg border text-left transition-all cursor-pointer"
                  :class="[
                    defaultEdgeType === 'smoothstep'
                      ? 'border-primary bg-primary-light/20 ring-1 ring-primary/30'
                      : 'border-border bg-surface hover:bg-surface-hover'
                  ]"
                  @click="setDefaultEdgeType('smoothstep')"
                >
                  <Sparkles class="w-4 h-4 text-primary shrink-0 mt-0.5" />
                  <div class="flex flex-col">
                    <span class="text-xs font-medium text-primary-text">平滑贝塞尔曲线 (Smoothstep)</span>
                    <span class="text-[10px] text-muted mt-0.5">柔和弧度渲染，图形运算开销低，响应迅捷</span>
                  </div>
                </button>
              </div>
            </div>
          </div>

          <!-- TAB 4: 关于与数据存储 (About & Storage) -->
          <div v-if="activeTab === 'about'" class="space-y-6">
            <div>
              <h2 class="text-base font-bold text-primary-text">关于与数据存储</h2>
              <p class="text-xs text-muted mt-0.5">查看 ColLink 运行环境、本地数据库文件与快照备份</p>
            </div>

            <!-- 4.1 产品版本信息 -->
            <div class="p-5 rounded-xl border border-border bg-surface shadow-2xs flex items-center justify-between">
              <div class="flex items-center gap-3.5">
                <div class="w-12 h-12 rounded-xl bg-gradient-to-tr from-blue-600 to-indigo-600 flex items-center justify-center text-white shadow-md shadow-blue-500/20">
                  <svg class="w-7 h-7" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <rect x="3" y="4" width="7" height="10" rx="1.5" stroke="currentColor" stroke-width="2"/>
                    <rect x="14" y="10" width="7" height="10" rx="1.5" stroke="currentColor" stroke-width="2"/>
                    <path d="M 10 9 C 12 9, 12 16, 14 16" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
                  </svg>
                </div>
                <div>
                  <div class="flex items-center gap-2">
                    <h3 class="text-sm font-bold text-primary-text font-sans">ColLink Desktop</h3>
                    <span class="px-2 py-0.5 rounded-full text-[10px] font-mono bg-primary-light text-primary font-semibold border border-primary-border">
                      v1.0.0
                    </span>
                  </div>
                  <p class="text-xs text-muted mt-1">企业级数据血缘分析、字段映射拓扑与智能协同架构平台</p>
                </div>
              </div>

              <div class="text-right text-[11px] text-muted">
                <p>架构：Tauri v2 + Vue 3 + SQLite</p>
                <p class="mt-0.5">内核：Rust 2021 Edition</p>
              </div>
            </div>

            <!-- 4.2 本地 SQLite 数据库路径 -->
            <div class="p-4 rounded-xl border border-border bg-surface shadow-2xs space-y-3">
              <div class="flex items-center justify-between">
                <div>
                  <h3 class="text-xs font-semibold text-primary-text">本地 SQLite 数据库文件</h3>
                  <p class="text-[11px] text-muted mt-0.5">所有项目空间、血缘图谱与字段元数据均持久化在本地单文件数据库中</p>
                </div>
                <button
                  type="button"
                  class="px-2.5 py-1 text-xs rounded border border-border bg-surface hover:bg-surface-hover text-primary transition-colors cursor-pointer flex items-center gap-1.5"
                  @click="openDbDirectory"
                >
                  <FolderGit2 class="w-3.5 h-3.5" />
                  <span>打开所在文件夹</span>
                </button>
              </div>

              <div class="flex items-center gap-2 p-2 rounded-lg bg-surface-subtle border border-border">
                <span class="text-xs font-mono text-primary-text truncate select-all flex-1" :title="appInfo?.db_path">
                  {{ appInfo?.db_path || '正在读取本地数据库物理路径...' }}
                </span>
                <button
                  type="button"
                  class="px-2 py-1 text-xs rounded border border-border bg-surface hover:bg-surface-hover text-muted hover:text-primary-text transition-colors shrink-0 cursor-pointer flex items-center gap-1"
                  @click="copyText(appInfo?.db_path || '', '数据库绝对路径已复制')"
                >
                  <Copy class="w-3 h-3" />
                  <span>复制路径</span>
                </button>
              </div>
            </div>

            <!-- 4.3 数据备份与迁移操作 -->
            <div class="p-4 rounded-xl border border-border bg-surface shadow-2xs flex items-center justify-between">
              <div>
                <h3 class="text-xs font-semibold text-primary-text">项目数据快照备份与迁移</h3>
                <p class="text-[11px] text-muted mt-0.5">导出当前项目全量结构拓扑为通用 JSON，或在不同环境之间恢复工程快照</p>
              </div>

              <button
                type="button"
                class="px-3 py-1.5 rounded-lg border border-border bg-surface hover:bg-surface-hover text-xs font-medium text-primary-text transition-colors cursor-pointer flex items-center gap-1.5"
                @click="$emit('open-backup')"
              >
                <Archive class="w-3.5 h-3.5 text-primary" />
                <span>打开备份管理面板</span>
              </button>
            </div>
          </div>

        </div>
      </div>
    </div>
  </div>
</template>

<script setup lang="ts">
import { ref, computed, onMounted } from 'vue'
import {
  Settings,
  ChevronLeft,
  Minus,
  Square,
  Copy,
  X,
  Palette,
  Cpu,
  Sliders,
  Info,
  Sun,
  Moon,
  Monitor,
  Check,
  Route,
  Sparkles,
  FolderGit2,
  Archive,
} from 'lucide-vue-next'
import { invoke } from '@tauri-apps/api/core'
import { useThemeStore, ACCENT_COLORS } from '@/stores/themeStore'
import { api } from '@/api'
import { toast } from '@/utils/toast'

const emit = defineEmits(['back', 'open-backup'])

const themeStore = useThemeStore()

// Tauri frameless window state
const isTauri = ref(typeof window !== 'undefined' && '__TAURI_INTERNALS__' in window)
const isMaximized = ref(false)

const checkMaximized = async () => {
  if (isTauri.value) {
    try {
      isMaximized.value = await invoke<boolean>('window_is_maximized')
    } catch {
      // ignore
    }
  }
}

onMounted(() => {
  checkMaximized()
  loadAppInfo()
})

const handleMinimize = async () => {
  try {
    await invoke('window_minimize')
  } catch (e) {
    console.error('Failed to minimize window:', e)
  }
}

const handleToggleMaximize = async () => {
  try {
    const res = await invoke<boolean>('window_toggle_maximize')
    isMaximized.value = res
  } catch (e) {
    console.error('Failed to toggle maximize window:', e)
  }
}

const handleClose = async () => {
  try {
    await invoke('window_close')
  } catch (e) {
    console.error('Failed to close window:', e)
  }
}

// Navigation Tabs
const activeTab = ref<'appearance' | 'mcp' | 'general' | 'about'>('appearance')

const navItems = [
  { id: 'appearance', label: '外观与主题', subLabel: '深浅模式、全站强调色与网格', icon: Palette },
  { id: 'mcp', label: 'MCP 协同与 AI', subLabel: 'Claude / Cursor 一键配置', icon: Cpu },
  { id: 'general', label: '常规与偏好', subLabel: '托盘驻留、提示与连线算法', icon: Sliders },
  { id: 'about', label: '关于与数据', subLabel: '版本说明与 SQLite 物理路径', icon: Info },
]

// System Info
const appInfo = ref<{ db_path: string; mcp_path: string; app_version: string } | null>(null)

const loadAppInfo = async () => {
  try {
    appInfo.value = await api.getAppInfo()
  } catch (e) {
    console.error('Failed to get app info:', e)
  }
}

const copyText = async (text: string, successMsg: string) => {
  if (!text) return
  try {
    await navigator.clipboard.writeText(text)
    toast.success(successMsg)
  } catch (err) {
    toast.error('复制失败，请手动选取复制')
  }
}

const openDbDirectory = async () => {
  if (appInfo.value?.db_path) {
    try {
      await api.openPathInExplorer(appInfo.value.db_path)
    } catch (e) {
      toast.error('无法打开目录: ' + e)
    }
  }
}

// MCP Client Selector
const activeMcpClient = ref<'claude' | 'cursor' | 'antigravity'>('claude')

const mcpClients = [
  { id: 'claude', name: 'Claude Desktop' },
  { id: 'cursor', name: 'Cursor IDE' },
  { id: 'antigravity', name: 'Antigravity / 通用' },
]

const activeClientConfigJson = computed(() => {
  const exePath = appInfo.value?.mcp_path ? appInfo.value.mcp_path.replace(/\\/g, '\\\\') : 'C:\\\\path\\\\to\\\\collink-mcp.exe'

  if (activeMcpClient.value === 'claude') {
    return JSON.stringify(
      {
        mcpServers: {
          collink: {
            command: exePath,
            args: []
          }
        }
      },
      null,
      2
    )
  } else if (activeMcpClient.value === 'cursor') {
    return JSON.stringify(
      {
        mcpServers: {
          collink: {
            command: exePath,
            args: [],
            transport: 'stdio'
          }
        }
      },
      null,
      2
    )
  } else {
    return JSON.stringify(
      {
        name: 'collink-mcp',
        type: 'stdio',
        command: exePath,
        args: []
      },
      null,
      2
    )
  }
})

// MCP Tools List
const mcpToolList = [
  { name: 'list_tables', category: '元数据', description: '获取当前项目所有表名、字段数及业务注释' },
  { name: 'inspect_table', category: '元数据', description: '深入查询单张数据表的全部字段名、数据类型与注释' },
  { name: 'get_lineage_path', category: '血缘链路', description: '全链路追溯指定字段的全部上下游来源与归宿节点' },
  { name: 'search_mappings', category: '血缘链路', description: '按关键词模糊检索当前画布中的字段对应映射' },
  { name: 'discover_relation_path', category: '智能推理', description: '基于外键与字段同义词智能推导两张表间的最优路径' },
  { name: 'export_lineage_graph', category: '拓扑导出', description: '以标准 JSON 结构导出画布全量血缘拓扑关系图谱' },
  { name: 'list_projects', category: '工程空间', description: '列出系统内已创建的所有项目工作空间' },
  { name: 'get_project_detail', category: '工程空间', description: '获取指定项目空间的详情与所包含的主题画板' },
]

// General Settings State
const closeToTray = ref(localStorage.getItem('collink-close-to-tray') !== 'false')
const setCloseToTray = (val: boolean) => {
  closeToTray.value = val
  localStorage.setItem('collink-close-to-tray', val ? 'true' : 'false')
  toast.success(val ? '已设置：关闭时最小化至后台托盘' : '已设置：关闭时彻底退出程序')
}

const showFloatingTip = ref(localStorage.getItem('collink_show_floating_tip') !== 'false')
const toggleFloatingTip = () => {
  showFloatingTip.value = !showFloatingTip.value
  localStorage.setItem('collink_show_floating_tip', showFloatingTip.value ? 'true' : 'false')
  toast.success(showFloatingTip.value ? '已开启画布操作提示' : '已隐藏画布操作提示')
}

const defaultEdgeType = ref(localStorage.getItem('collink-default-edge-type') || 'smartObstacle')
const setDefaultEdgeType = (type: string) => {
  defaultEdgeType.value = type
  localStorage.setItem('collink-default-edge-type', type)
  toast.success('连线算法偏好已保存')
}
</script>

<style scoped>
.custom-scrollbar::-webkit-scrollbar {
  width: 6px;
  height: 6px;
}
.custom-scrollbar::-webkit-scrollbar-thumb {
  background: var(--cl-border);
  border-radius: 9999px;
}
.custom-scrollbar::-webkit-scrollbar-thumb:hover {
  background: var(--cl-border-strong);
}
</style>
