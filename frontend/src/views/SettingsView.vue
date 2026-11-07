<template>
  <div class="h-screen w-screen flex flex-col bg-surface text-primary-text select-none overflow-hidden font-sans">
    <!-- Top Header Bar with Native Drag Region & Window Controls -->
    <div
      data-tauri-drag-region
      class="h-11 pl-3 pr-0 bg-surface border-b border-border flex items-center justify-between z-50 text-sm shrink-0 shadow-2xs"
    >
      <!-- Left: Back Button & Title -->
      <div class="flex items-center gap-2.5">
        <button
          type="button"
          class="flex items-center gap-1.5 px-3 py-1.5 rounded-md text-sm font-medium text-muted hover:text-primary-text hover:bg-surface-hover border border-border transition-colors cursor-pointer"
          title="返回工作区画布 (Esc / Ctrl+,)"
          @click="$emit('back')"
        >
          <ChevronLeft class="w-4 h-4" />
          <span>返回画板</span>
        </button>

        <div class="w-px h-4 bg-border mx-1"></div>

        <div class="flex items-center gap-1.5 text-sm text-primary-text font-semibold">
          <Settings class="w-4 h-4 text-primary" />
          <span>系统设置</span>
          <span class="text-xs text-muted font-normal ml-1">Settings</span>
        </div>
      </div>

      <!-- Right: Window Controls (Tauri Custom Titlebar) -->
      <div v-if="isTauri" class="flex items-center h-11 select-none">
        <button
          type="button"
          class="w-11 h-11 flex items-center justify-center text-muted hover:text-primary-text hover:bg-surface-hover transition-colors cursor-pointer"
          title="最小化"
          @click="handleMinimize"
        >
          <Minus class="w-4 h-4" />
        </button>

        <button
          type="button"
          class="w-11 h-11 flex items-center justify-center text-muted hover:text-primary-text hover:bg-surface-hover transition-colors cursor-pointer"
          :title="isMaximized ? '向下还原' : '最大化'"
          @click="handleToggleMaximize"
        >
          <Copy v-if="isMaximized" class="w-3.5 h-3.5 rotate-180" />
          <Square v-else class="w-3.5 h-3.5" />
        </button>

        <button
          type="button"
          class="w-11 h-11 flex items-center justify-center text-muted hover:text-white hover:bg-red-600 transition-colors cursor-pointer"
          title="隐藏到托盘 / 关闭"
          @click="handleClose"
        >
          <X class="w-4 h-4" />
        </button>
      </div>
    </div>

    <!-- Main Content Area: Split Sidebar & Settings Panel -->
    <div class="flex-1 flex min-h-0 overflow-hidden">
      <!-- Left Sidebar Navigation -->
      <div class="w-64 border-r border-border bg-surface-subtle/40 p-3.5 flex flex-col gap-1 shrink-0 overflow-y-auto">
        <div class="px-2.5 py-1.5 text-xs font-semibold text-muted tracking-wider uppercase">
          设置分类
        </div>

        <button
          v-for="item in navItems"
          :key="item.id"
          type="button"
          class="flex items-center gap-3 px-3.5 py-2.5 rounded-lg text-sm font-medium transition-all duration-150 cursor-pointer text-left"
          :class="[
            activeTab === item.id
              ? 'bg-primary-light/50 text-primary font-semibold shadow-2xs'
              : 'text-muted hover:text-primary-text hover:bg-surface-hover/80'
          ]"
          @click="activeTab = item.id"
        >
          <component :is="item.icon" class="w-4.5 h-4.5 shrink-0" :class="activeTab === item.id ? 'text-primary' : 'text-muted'" />
          <div class="flex flex-col min-w-0">
            <span class="leading-tight text-sm">{{ item.label }}</span>
            <span class="text-xs text-muted truncate mt-0.5">{{ item.subLabel }}</span>
          </div>
        </button>

        <div class="mt-auto pt-3 border-t border-border">
          <div class="px-2.5 py-1.5 text-xs text-muted flex items-center justify-between">
            <span>快捷键返回</span>
            <kbd class="px-1.5 py-0.5 rounded bg-surface border border-border text-xs font-mono">Esc</kbd>
          </div>
        </div>
      </div>

      <!-- Right Settings Content Panel -->
      <div class="flex-1 overflow-y-auto p-8 lg:p-10 bg-surface-subtle/20 custom-scrollbar">
        <div class="max-w-4xl mx-auto space-y-7">

          <!-- TAB 1: 外观与主题 (Appearance & Theme) -->
          <div v-if="activeTab === 'appearance'" class="space-y-7">
            <div>
              <h2 class="text-xl font-bold text-primary-text">外观与主题</h2>
              <p class="text-sm text-muted mt-1">自定义界面视觉风格、主题强调色与画布背景表现</p>
            </div>

            <!-- 1.1 主题模式 -->
            <div class="p-5 rounded-xl border border-border bg-surface shadow-2xs space-y-4">
              <div class="flex items-center justify-between">
                <div>
                  <h3 class="text-sm font-semibold text-primary-text">显示模式</h3>
                  <p class="text-xs text-muted mt-0.5">选择浅色、深色或随系统自动切换</p>
                </div>
              </div>

              <div class="grid grid-cols-3 gap-3.5 pt-1">
                <button
                  type="button"
                  class="flex flex-col items-center justify-center gap-2.5 p-4 rounded-lg border text-sm font-medium transition-all cursor-pointer"
                  :class="[
                    themeStore.mode === 'light'
                      ? 'border-primary bg-primary-light/30 text-primary ring-1 ring-primary/40'
                      : 'border-border bg-surface hover:bg-surface-hover text-muted hover:text-primary-text'
                  ]"
                  @click="themeStore.setMode('light', $event)"
                >
                  <Sun class="w-6 h-6 text-amber-500" />
                  <span>浅色模式</span>
                </button>

                <button
                  type="button"
                  class="flex flex-col items-center justify-center gap-2.5 p-4 rounded-lg border text-sm font-medium transition-all cursor-pointer"
                  :class="[
                    themeStore.mode === 'dark'
                      ? 'border-primary bg-primary-light/30 text-primary ring-1 ring-primary/40'
                      : 'border-border bg-surface hover:bg-surface-hover text-muted hover:text-primary-text'
                  ]"
                  @click="themeStore.setMode('dark', $event)"
                >
                  <Moon class="w-6 h-6 text-indigo-400" />
                  <span>深色模式</span>
                </button>

                <button
                  type="button"
                  class="flex flex-col items-center justify-center gap-2.5 p-4 rounded-lg border text-sm font-medium transition-all cursor-pointer"
                  :class="[
                    themeStore.mode === 'auto'
                      ? 'border-primary bg-primary-light/30 text-primary ring-1 ring-primary/40'
                      : 'border-border bg-surface hover:bg-surface-hover text-muted hover:text-primary-text'
                  ]"
                  @click="themeStore.setMode('auto', $event)"
                >
                  <Monitor class="w-6 h-6 text-blue-500" />
                  <span>跟随系统</span>
                </button>
              </div>
            </div>

            <!-- 1.2 主题强调色 (同步控制画布连线与交互) -->
            <div class="p-5 rounded-xl border border-border bg-surface shadow-2xs space-y-4">
              <div class="flex items-center justify-between">
                <div>
                  <h3 class="text-sm font-semibold text-primary-text">主题强调色</h3>
                  <p class="text-xs text-muted mt-0.5">控制全站按钮焦点、边框以及画板连线高亮与脉冲动效</p>
                </div>
                <!-- Preview Tag -->
                <div class="flex items-center gap-1.5 px-2.5 py-1 rounded border border-primary bg-primary-light/50 text-primary text-xs font-medium">
                  <span class="w-2 h-2 rounded-full bg-primary animate-pulse"></span>
                  <span>当前强调色</span>
                </div>
              </div>

              <div class="grid grid-cols-4 gap-3 pt-1">
                <button
                  v-for="color in ACCENT_COLORS"
                  :key="color.id"
                  type="button"
                  class="flex items-center gap-3 p-2.5 rounded-lg border text-sm transition-all cursor-pointer text-left group"
                  :class="[
                    themeStore.accentColorId === color.id
                      ? 'border-primary bg-primary-light/20 ring-1 ring-primary/30 font-medium'
                      : 'border-border bg-surface hover:bg-surface-hover'
                  ]"
                  @click="themeStore.setAccentColor(color.id)"
                >
                  <span
                    class="w-5 h-5 rounded-full shrink-0 shadow-xs flex items-center justify-center text-white text-[10px]"
                    :style="{ backgroundColor: color.hex }"
                  >
                    <Check v-if="themeStore.accentColorId === color.id" class="w-3.5 h-3.5 stroke-[3]" />
                  </span>
                  <div class="flex flex-col min-w-0">
                    <span class="text-sm text-primary-text truncate leading-tight font-medium">{{ color.name }}</span>
                    <span class="text-xs text-muted font-mono leading-tight mt-0.5">{{ color.hex }}</span>
                  </div>
                </button>
              </div>
            </div>

            <!-- 1.3 画布底纹网格 -->
            <div class="p-5 rounded-xl border border-border bg-surface shadow-2xs space-y-4">
              <div>
                <h3 class="text-sm font-semibold text-primary-text">画板背景网格</h3>
                <p class="text-xs text-muted mt-0.5">设置画板工作区的背景纹理样式</p>
              </div>

              <div class="grid grid-cols-2 gap-3.5 pt-1">
                <button
                  type="button"
                  class="flex items-center gap-3.5 p-3.5 rounded-lg border text-sm transition-all cursor-pointer"
                  :class="[
                    themeStore.gridPattern === 'dots'
                      ? 'border-primary bg-primary-light/20 text-primary font-medium ring-1 ring-primary/30'
                      : 'border-border bg-surface hover:bg-surface-hover text-muted hover:text-primary-text'
                  ]"
                  @click="themeStore.setGridPattern('dots')"
                >
                  <div class="w-10 h-10 rounded border border-border bg-surface-subtle flex items-center justify-center shrink-0">
                    <div class="grid grid-cols-2 gap-1.5">
                      <span class="w-1.5 h-1.5 rounded-full bg-muted"></span>
                      <span class="w-1.5 h-1.5 rounded-full bg-muted"></span>
                      <span class="w-1.5 h-1.5 rounded-full bg-muted"></span>
                      <span class="w-1.5 h-1.5 rounded-full bg-muted"></span>
                    </div>
                  </div>
                  <div class="flex flex-col text-left">
                    <span class="text-sm text-primary-text font-semibold">点阵网格 (Dots)</span>
                    <span class="text-xs text-muted mt-0.5">简洁通透，突出数据卡片</span>
                  </div>
                </button>

                <button
                  type="button"
                  class="flex items-center gap-3.5 p-3.5 rounded-lg border text-sm transition-all cursor-pointer"
                  :class="[
                    themeStore.gridPattern === 'lines'
                      ? 'border-primary bg-primary-light/20 text-primary font-medium ring-1 ring-primary/30'
                      : 'border-border bg-surface hover:bg-surface-hover text-muted hover:text-primary-text'
                  ]"
                  @click="themeStore.setGridPattern('lines')"
                >
                  <div class="w-10 h-10 rounded border border-border bg-surface-subtle flex items-center justify-center shrink-0">
                    <div class="w-5 h-5 border border-dashed border-muted"></div>
                  </div>
                  <div class="flex flex-col text-left">
                    <span class="text-sm text-primary-text font-semibold">方格线条 (Lines)</span>
                    <span class="text-xs text-muted mt-0.5">结构清晰，便于对齐排版</span>
                  </div>
                </button>
              </div>
            </div>
          </div>

          <!-- TAB 2: MCP 协同与 AI (Model Context Protocol) -->
          <div v-if="activeTab === 'mcp'" class="space-y-7">
            <div>
              <h2 class="text-xl font-bold text-primary-text">MCP 协同</h2>
              <p class="text-sm text-muted mt-1">接入 Claude Desktop、Cursor、Antigravity 等支持 MCP 协议的 AI 助手</p>
            </div>

            <!-- 2.1 服务核心路径与状态 -->
            <div class="p-5 rounded-xl border border-border bg-surface shadow-2xs space-y-4">
              <div class="flex items-center justify-between">
                <div class="flex items-center gap-2.5">
                  <div class="w-7 h-7 rounded-md bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 flex items-center justify-center shrink-0 border border-emerald-500/20">
                    <Cpu class="w-4 h-4" />
                  </div>
                  <div>
                    <h3 class="text-sm font-semibold text-primary-text">服务运行状态</h3>
                    <p class="text-xs text-muted mt-0.5">基于 JSON-RPC 2.0 与标准输入输出 (stdio) 通信</p>
                  </div>
                </div>
                <span class="inline-flex items-center gap-1.5 px-2.5 py-1 rounded-full text-xs font-medium bg-emerald-500/10 text-emerald-600 dark:text-emerald-400 border border-emerald-500/20">
                  <span class="w-1.5 h-1.5 rounded-full bg-emerald-500 animate-pulse"></span>
                  服务就绪 (Ready)
                </span>
              </div>

              <!-- Executable Path Box -->
              <div class="flex items-center gap-2.5 p-2.5 rounded-lg bg-surface-subtle border border-border">
                <div class="flex flex-col min-w-0 flex-1">
                  <span class="text-xs text-muted font-medium">MCP 服务执行路径 (collink-mcp.exe):</span>
                  <span class="text-sm font-mono text-primary-text truncate select-all mt-0.5" :title="cleanMcpPath">
                    {{ cleanMcpPath || '正在读取可执行路径...' }}
                  </span>
                </div>
                <button
                  type="button"
                  class="px-3 py-1.5 text-xs font-medium rounded border border-border bg-surface hover:bg-surface-hover text-muted hover:text-primary-text transition-colors shrink-0 cursor-pointer flex items-center gap-1.5"
                  @click="copyText(cleanMcpPath || '', 'MCP 服务路径已复制')"
                >
                  <Copy class="w-3.5 h-3.5" />
                  <span>复制路径</span>
                </button>
              </div>
            </div>

            <!-- 2.2 客户端接入配置一键生成器 -->
            <div class="p-5 rounded-xl border border-border bg-surface shadow-2xs space-y-4">
              <div class="flex items-center justify-between">
                <div>
                  <h3 class="text-sm font-semibold text-primary-text">客户端接入配置</h3>
                  <p class="text-xs text-muted mt-0.5">将生成的 JSON 片段直接粘贴至对应客户端的配置文件中</p>
                </div>
                <!-- Client Switcher Tabs -->
                <div class="flex items-center p-0.5 rounded-lg bg-surface-subtle border border-border text-xs">
                  <button
                    v-for="client in mcpClients"
                    :key="client.id"
                    type="button"
                    class="px-3 py-1.5 rounded-md transition-all cursor-pointer text-xs font-medium"
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
              <div class="relative rounded-lg border border-border bg-canvas p-3.5 font-mono text-xs overflow-x-auto">
                <pre class="text-primary-text text-xs leading-relaxed select-all"><code>{{ activeClientConfigJson }}</code></pre>
                <div class="absolute top-2.5 right-2.5 flex items-center gap-1.5">
                  <button
                    type="button"
                    class="px-3 py-1.5 rounded border border-border bg-surface hover:bg-surface-hover text-primary-text text-xs font-medium transition-colors cursor-pointer flex items-center gap-1.5 shadow-2xs"
                    @click="copyText(activeClientConfigJson, 'JSON 配置代码已复制到剪贴板')"
                  >
                    <Copy class="w-3.5 h-3.5 text-muted" />
                    <span>复制配置</span>
                  </button>
                </div>
              </div>
            </div>

            <!-- 2.3 内建 8 大 MCP 工具能力 -->
            <div class="p-5 rounded-xl border border-border bg-surface shadow-2xs space-y-4">
              <div>
                <h3 class="text-sm font-semibold text-primary-text">内置工具清单 (8 个 Tools)</h3>
                <p class="text-xs text-muted mt-0.5">AI 助手接入后可调用的元数据检索与血缘分析工具</p>
              </div>

              <div class="grid grid-cols-2 gap-3">
                <div
                  v-for="tool in mcpToolList"
                  :key="tool.name"
                  class="p-3 rounded-lg border border-border bg-surface-subtle/30 space-y-1.5 text-left"
                >
                  <div class="flex items-center justify-between">
                    <span class="text-sm font-mono font-semibold text-primary">{{ tool.name }}</span>
                    <span class="text-[10px] px-1.5 py-0.5 rounded bg-surface border border-border text-muted font-mono">{{ tool.category }}</span>
                  </div>
                  <p class="text-xs text-muted leading-snug">{{ tool.description }}</p>
                </div>
              </div>
            </div>
          </div>

          <!-- TAB 3: 常规设置 (General & Canvas) -->
          <div v-if="activeTab === 'general'" class="space-y-7">
            <div>
              <h2 class="text-xl font-bold text-primary-text">常规设置</h2>
              <p class="text-sm text-muted mt-1">运行行为偏好与画板交互配置</p>
            </div>

            <!-- 3.1 窗口关闭行为 -->
            <div class="p-5 rounded-xl border border-border bg-surface shadow-2xs space-y-4">
              <div>
                <h3 class="text-sm font-semibold text-primary-text">窗口关闭行为</h3>
                <p class="text-xs text-muted mt-0.5">自定义点击右上角关闭按钮或快捷键关闭时的动作</p>
              </div>

              <div class="space-y-2.5">
                <label
                  class="flex items-start gap-3.5 p-3.5 rounded-lg border cursor-pointer transition-colors"
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
                    class="mt-1 text-primary focus:ring-primary w-4 h-4"
                    @change="setCloseToTray(true)"
                  />
                  <div class="flex flex-col">
                    <span class="text-sm font-medium text-primary-text">最小化到系统托盘并在后台驻留（推荐）</span>
                    <span class="text-xs text-muted mt-1">关闭窗口时不中断进程，可随时通过托盘图标快速唤起</span>
                  </div>
                </label>

                <label
                  class="flex items-start gap-3.5 p-3.5 rounded-lg border cursor-pointer transition-colors"
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
                    class="mt-1 text-primary focus:ring-primary w-4 h-4"
                    @change="setCloseToTray(false)"
                  />
                  <div class="flex flex-col">
                    <span class="text-sm font-medium text-primary-text">直接退出程序</span>
                    <span class="text-xs text-muted mt-1">关闭窗口并结束所有进程，释放系统资源</span>
                  </div>
                </label>
              </div>
            </div>

            <!-- 3.2 画布左下角操作提示 -->
            <div class="p-5 rounded-xl border border-border bg-surface shadow-2xs flex items-center justify-between">
              <div>
                <h3 class="text-sm font-semibold text-primary-text">画板操作提示</h3>
                <p class="text-xs text-muted mt-0.5">在工作区左下角显示血缘聚焦、连线与脉冲快捷操作指引</p>
              </div>

              <button
                type="button"
                class="w-11 h-6 rounded-full transition-colors relative cursor-pointer focus:outline-none"
                :class="showFloatingTip ? 'bg-primary' : 'bg-surface-hover border border-border'"
                @click="toggleFloatingTip"
              >
                <span
                  class="w-4 h-4 rounded-full bg-white shadow-xs absolute top-1 transition-transform"
                  :class="showFloatingTip ? 'right-1' : 'left-1'"
                ></span>
              </button>
            </div>

            <!-- 3.3 默认连线算法 -->
            <div class="p-5 rounded-xl border border-border bg-surface shadow-2xs space-y-4">
              <div>
                <h3 class="text-sm font-semibold text-primary-text">连线路由算法</h3>
                <p class="text-xs text-muted mt-0.5">设置字段映射连线的折线避障或平滑曲线算法</p>
              </div>

              <div class="grid grid-cols-2 gap-3.5 pt-1">
                <button
                  type="button"
                  class="flex items-start gap-3.5 p-3.5 rounded-lg border text-left transition-all cursor-pointer"
                  :class="[
                    defaultEdgeType === 'smartObstacle'
                      ? 'border-primary bg-primary-light/20 ring-1 ring-primary/30'
                      : 'border-border bg-surface hover:bg-surface-hover'
                  ]"
                  @click="setDefaultEdgeType('smartObstacle')"
                >
                  <Route class="w-5 h-5 text-primary shrink-0 mt-0.5" />
                  <div class="flex flex-col">
                    <span class="text-sm font-semibold text-primary-text">智能避障折线 (Smart Obstacle)</span>
                    <span class="text-xs text-muted mt-1">自动绕行数据卡片，保持连线工整不穿插</span>
                  </div>
                </button>

                <button
                  type="button"
                  class="flex items-start gap-3.5 p-3.5 rounded-lg border text-left transition-all cursor-pointer"
                  :class="[
                    defaultEdgeType === 'smoothstep'
                      ? 'border-primary bg-primary-light/20 ring-1 ring-primary/30'
                      : 'border-border bg-surface hover:bg-surface-hover'
                  ]"
                  @click="setDefaultEdgeType('smoothstep')"
                >
                  <Sparkles class="w-5 h-5 text-primary shrink-0 mt-0.5" />
                  <div class="flex flex-col">
                    <span class="text-sm font-semibold text-primary-text">平滑曲线 (Smoothstep)</span>
                    <span class="text-xs text-muted mt-1">顺滑弧线渲染，视觉流动感更强</span>
                  </div>
                </button>
              </div>
            </div>
          </div>

          <!-- TAB 4: 关于与数据 (About & Storage) -->
          <div v-if="activeTab === 'about'" class="space-y-7">
            <div>
              <h2 class="text-xl font-bold text-primary-text">关于与数据</h2>
              <p class="text-sm text-muted mt-1">软件版本信息与本地数据文件管理</p>
            </div>

            <!-- 4.1 产品版本信息 -->
            <div class="p-6 rounded-xl border border-border bg-surface shadow-2xs flex items-center justify-between">
              <div class="flex items-center gap-4">
                <div class="w-14 h-14 rounded-2xl bg-gradient-to-tr from-blue-600 to-indigo-600 flex items-center justify-center text-white shadow-md shadow-blue-500/20 shrink-0">
                  <svg class="w-8 h-8" viewBox="0 0 24 24" fill="none" xmlns="http://www.w3.org/2000/svg">
                    <rect x="3" y="4" width="7" height="10" rx="1.5" stroke="currentColor" stroke-width="2"/>
                    <rect x="14" y="10" width="7" height="10" rx="1.5" stroke="currentColor" stroke-width="2"/>
                    <path d="M 10 9 C 12 9, 12 16, 14 16" stroke="currentColor" stroke-width="2" stroke-linecap="round"/>
                  </svg>
                </div>
                <div>
                  <div class="flex items-center gap-2.5">
                    <h3 class="text-base font-bold text-primary-text font-sans">ColLink Desktop</h3>
                    <span class="px-2.5 py-0.5 rounded-full text-xs font-mono bg-primary-light text-primary font-semibold border border-primary-border">
                      v1.0.0
                    </span>
                  </div>
                  <p class="text-sm text-muted mt-1.5">数据血缘分析、字段映射拓扑与智能协同平台</p>
                </div>
              </div>

              <div class="text-right text-xs text-muted space-y-1">
                <p>架构：Tauri v2 + Vue 3</p>
                <p>内核：Rust 2021 Edition</p>
              </div>
            </div>

            <!-- 4.2 本地 SQLite 数据库路径 -->
            <div class="p-5 rounded-xl border border-border bg-surface shadow-2xs space-y-4">
              <div class="flex items-center justify-between">
                <div>
                  <h3 class="text-sm font-semibold text-primary-text">本地数据存储</h3>
                  <p class="text-xs text-muted mt-0.5">项目空间、画板拓扑与字段映射保存在本地单文件 SQLite 数据库中</p>
                </div>
                <button
                  type="button"
                  class="px-3 py-1.5 text-xs font-medium rounded border border-border bg-surface hover:bg-surface-hover text-primary transition-colors cursor-pointer flex items-center gap-1.5"
                  @click="openDbDirectory"
                >
                  <FolderGit2 class="w-4 h-4" />
                  <span>打开所在文件夹</span>
                </button>
              </div>

              <div class="flex items-center gap-2.5 p-2.5 rounded-lg bg-surface-subtle border border-border">
                <span class="text-sm font-mono text-primary-text truncate select-all flex-1" :title="cleanDbPath">
                  {{ cleanDbPath || '正在读取本地数据库物理路径...' }}
                </span>
                <button
                  type="button"
                  class="px-3 py-1.5 text-xs font-medium rounded border border-border bg-surface hover:bg-surface-hover text-muted hover:text-primary-text transition-colors shrink-0 cursor-pointer flex items-center gap-1.5"
                  @click="copyText(cleanDbPath || '', '数据库路径已复制')"
                >
                  <Copy class="w-3.5 h-3.5" />
                  <span>复制路径</span>
                </button>
              </div>
            </div>

            <!-- 4.3 数据备份与迁移操作 -->
            <div class="p-5 rounded-xl border border-border bg-surface shadow-2xs flex items-center justify-between">
              <div>
                <h3 class="text-sm font-semibold text-primary-text">项目数据备份与恢复</h3>
                <p class="text-xs text-muted mt-0.5">导出项目全量拓扑数据为 JSON 文件，或从备份快照导入</p>
              </div>

              <button
                type="button"
                class="px-4 py-2 rounded-lg border border-border bg-surface hover:bg-surface-hover text-sm font-medium text-primary-text transition-colors cursor-pointer flex items-center gap-2"
                @click="$emit('open-backup')"
              >
                <Archive class="w-4 h-4 text-primary" />
                <span>打开备份管理</span>
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
  { id: 'appearance', label: '外观与主题', subLabel: '主题模式、强调色与网格', icon: Palette },
  { id: 'mcp', label: 'MCP 协同', subLabel: 'AI 编程助手接入配置', icon: Cpu },
  { id: 'general', label: '常规设置', subLabel: '运行行为与画布偏好', icon: Sliders },
  { id: 'about', label: '关于与数据', subLabel: '版本信息与存储路径', icon: Info },
]

// System Info
const appInfo = ref<{ db_path: string; mcp_path: string; app_version: string } | null>(null)

const cleanDbPath = computed(() => (appInfo.value?.db_path || '').replace(/^\\\\\?\\/, ''))
const cleanMcpPath = computed(() => (appInfo.value?.mcp_path || '').replace(/^\\\\\?\\/, ''))

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
  const p = cleanDbPath.value || appInfo.value?.db_path
  if (p) {
    try {
      await api.openPathInExplorer(p)
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
  const rawPath = cleanMcpPath.value || appInfo.value?.mcp_path || 'C:\\path\\to\\collink-mcp.exe'
  const exePath = rawPath.replace(/\\/g, '\\\\')

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
