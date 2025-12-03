# 🚀 Verdent AI 账号管理器

<div align="center">
  <img src="Verdent_account_manger/public/verdent-long.svg" alt="Verdent Logo" width="400"/>
  
  [![License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)
  [![GitHub Actions](https://img.shields.io/badge/CI/CD-GitHub_Actions-2088FF.svg)](https://github.com/features/actions)
  [![Tauri](https://img.shields.io/badge/Built_with-Tauri-FFC131.svg)](https://tauri.app/)
  [![Vue 3](https://img.shields.io/badge/Vue-3.x-4FC08D.svg)](https://vuejs.org/)
  [![Rust](https://img.shields.io/badge/Rust-stable-orange.svg)](https://www.rust-lang.org/)
</div>

## 📋 目录

- [功能特性](#-功能特性)
- [系统要求](#-系统要求)
- [快速开始](#-快速开始)
- [使用指南](#-使用指南)
- [项目结构](#-项目结构)
- [开发指南](#-开发指南)
- [构建打包](#-构建打包)
- [贡献指南](#-贡献指南)
- [常见问题](#-常见问题)
- [许可证](#-许可证)

---

## ✨ 功能特性

### 🎯 核心功能
- **账号管理** - 集中管理所有 Verdent AI 账号
- **一键登录** - 支持 VS Code、Cursor、Windsurf 编辑器快速登录
- **自动注册** - 批量自动注册新账号（使用免费临时邮箱）
- **批量操作** - 批量刷新、导出、删除账号
- **智能筛选** - 按额度状态快速筛选账号
- **隐私保护** - 邮箱隐私模式保护敏感信息
- **数据持久化** - 所有设置和账号数据自动保存

### 💡 特色功能
- 📊 **实时统计** - 账号状态和额度统计面板
- 🔄 **批量导入** - 支持 Token、账号密码、JSON 格式导入
- 📤 **多格式导出** - 支持 JSON、CSV、TXT 格式导出
- 🎨 **现代化 UI** - 采用苹果风格设计，界面美观易用
- 🔐 **安全存储** - 本地加密存储账号信息
- ⚡ **高性能** - 基于 Tauri + Rust，资源占用低

---

## 💻 系统要求

### 基础要求
- **Windows**: Windows 10 1803 或更高版本
- **macOS**: macOS 10.13 或更高版本
- **Linux**: Ubuntu 20.04 或更高版本（需要 WebKit2GTK）

### 运行环境
- **内存**: 最低 512MB（推荐 1GB+）
- **存储**: 100MB 可用空间
- **网络**: 需要互联网连接（用于账号验证和注册）

---

## 🚀 快速开始

### 方式一：下载预编译版本（推荐）

1. **下载安装包**
   - 访问 [Releases](https://github.com/chaogei/Verdent/releases) 页面
   - 下载对应平台的安装包：
     - Windows: `Verdent账号管理器_1.2.0_x64.msi` 或 `.exe`
     - macOS: `Verdent账号管理器_1.2.0.dmg`
     - Linux: `Verdent账号管理器_1.2.0_amd64.deb` 或 `.AppImage`

2. **安装应用**
   - Windows: 双击安装包，按提示安装
   - macOS: 双击 dmg 文件，拖动到 Applications
   - Linux: 
     ```bash
     # Debian/Ubuntu (.deb)
     sudo dpkg -i Verdent账号管理器_1.2.0_amd64.deb
     
     # AppImage
     chmod +x Verdent账号管理器_1.2.0.AppImage
     ./Verdent账号管理器_1.2.0.AppImage
     ```

3. **启动应用**
   - 从开始菜单或应用程序文件夹启动 "Verdent账号管理器"

### 方式二：从源码构建

```bash
# 1. 克隆仓库
git clone https://github.com/chaogei/Verdent.git
cd Verdent

# 2. 安装依赖
cd Verdent_account_manger
npm install

# 3. 开发模式运行
npm run tauri dev

# 4. 构建发布版本
npm run tauri build
```

---

## 📖 使用指南

### 1. 首次使用

启动应用后，默认进入**账户管理**界面：

- **自动注册账号**: 点击右上角 "自动注册" 按钮
- **导入已有账号**: 点击 "导入账号" 按钮，支持多种导入方式

### 2. 账号管理

#### 自动注册
1. 点击 "自动注册" 按钮
2. 设置注册数量（1-10）和并发数（1-5）
3. 可选择使用随机密码或固定密码
4. 点击 "开始注册" 自动批量注册

#### 导入账号
支持三种导入方式：
- **Token 导入**: 每行一个 Token
- **账号密码导入**: 格式 `邮箱:密码`
- **JSON 导入**: 支持批量 JSON 数据

#### 账号操作
- **刷新**: 更新账号额度和状态信息
- **编辑**: 修改账号信息
- **删除**: 移除账号
- **一键登录**: 快速登录到编辑器

### 3. 编辑器集成

支持一键登录到以下编辑器：
- **VS Code**: 点击账号卡片的 VS Code 按钮
- **Cursor**: 点击账号卡片的 Cursor 按钮  
- **Windsurf**: 点击账号卡片的 Windsurf 按钮

### 4. 批量操作

1. **全选/多选**: 
   - 点击 "全选" 按钮选择所有账号
   - 点击账号卡片选择单个账号
   - 使用 `Ctrl+A` 快捷键全选

2. **批量操作**:
   - 批量刷新：更新选中账号的信息
   - 批量导出：导出选中账号数据
   - 批量删除：删除选中账号

### 5. 账号筛选

点击统计面板的卡片快速筛选：
- **满额度账号**: 未使用的新账号
- **已消耗账号**: 已使用但还有剩余
- **0额度账号**: 无额度账号
- **异常账号**: 额度用尽的账号

### 6. 隐私模式

点击 "隐私模式" 按钮保护邮箱地址：
- 自动隐藏邮箱中间部分
- 编辑时显示原始邮箱

---

## 📁 项目结构

```
Verdent/
├── 📱 Verdent_account_manger/    # 主应用程序（Tauri + Vue 3）
│   ├── src/                      # 前端源码
│   │   ├── components/           # Vue 组件
│   │   ├── App.vue              # 主应用组件
│   │   └── style.css            # 全局样式
│   ├── src-tauri/               # 后端源码（Rust）
│   │   ├── src/                 # Rust 源文件
│   │   ├── icons/               # 应用图标
│   │   └── tauri.conf.json     # Tauri 配置
│   └── package.json             # 前端依赖
│
├── 🤖 verdent_auto_register.py   # 自动注册脚本
├── 📦 build/                      # 构建输出目录
├── 📚 docs/                       # 项目文档
├── 🧪 test/                       # 测试文件
│
├── 🔧 构建脚本
│   ├── build_all.ps1            # 一键构建所有
│   ├── build_tauri.ps1          # 构建 Tauri 应用
│   ├── build_python.ps1         # 构建 Python 脚本
│   └── setup_github_actions.ps1 # 设置 CI/CD
│
└── ⚙️ 配置文件
    ├── .github/workflows/        # GitHub Actions 配置
    ├── requirements.txt          # Python 依赖
    └── README.md                # 本文件
```

---

## 🛠️ 开发指南

### 环境准备

#### Windows 开发环境
```powershell
# 1. 安装 Node.js (v18+)
winget install OpenJS.NodeJS

# 2. 安装 Rust
winget install Rustlang.Rust.MSVC

# 3. 安装 pnpm (推荐)
npm install -g pnpm

# 4. 安装 Tauri CLI
cargo install tauri-cli
```

#### macOS 开发环境
```bash
# 1. 安装 Homebrew
/bin/bash -c "$(curl -fsSL https://raw.githubusercontent.com/Homebrew/install/HEAD/install.sh)"

# 2. 安装依赖
brew install node
brew install rust

# 3. 安装开发工具
npm install -g pnpm
cargo install tauri-cli
```

#### Linux 开发环境
```bash
# Ubuntu/Debian
sudo apt update
sudo apt install -y nodejs npm rust-all libwebkit2gtk-4.1-dev

# 安装开发工具
npm install -g pnpm
cargo install tauri-cli
```

### 开发流程

1. **安装项目依赖**
   ```bash
   cd Verdent_account_manger
   pnpm install
   ```

2. **启动开发服务器**
   ```bash
   pnpm tauri dev
   ```

3. **代码热重载**
   - 前端代码修改自动热更新
   - Rust 代码修改自动重新编译

4. **调试**
   - 前端：按 `F12` 打开开发者工具
   - 后端：使用 `println!` 或 `dbg!` 宏

### 代码规范

- **前端**: Vue 3 Composition API + TypeScript
- **后端**: Rust 2021 Edition
- **样式**: 采用苹果设计风格
- **图标**: 使用 SVG 格式

---

## 📦 构建打包

### 本地构建

#### Windows
```powershell
# 一键构建所有
.\build_all.ps1

# 或分别构建
.\build_tauri.ps1    # 构建主应用
.\build_python.ps1   # 构建注册脚本
```

#### macOS/Linux
```bash
cd Verdent_account_manger
npm run tauri build
```

### GitHub Actions 自动构建

项目已配置 GitHub Actions 自动构建所有平台：

1. **设置 CI/CD**
   ```powershell
   .\setup_github_actions.ps1
   ```

2. **触发构建**
   ```bash
   git tag v1.2.0
   git push origin v1.2.0
   ```

3. **下载构建产物**
   - 访问 GitHub Actions 页面
   - 下载对应平台的安装包

详见：[跨平台打包指南](docs/跨平台打包快速开始.md)

---

## 🤝 贡献指南

欢迎贡献代码！请遵循以下步骤：

1. **Fork 项目**
2. **创建功能分支** (`git checkout -b feature/AmazingFeature`)
3. **提交更改** (`git commit -m 'Add some AmazingFeature'`)
4. **推送到分支** (`git push origin feature/AmazingFeature`)
5. **提交 Pull Request**

### 提交规范

使用语义化提交信息：
- `feat:` 新功能
- `fix:` 修复 bug
- `docs:` 文档更新
- `style:` 代码格式调整
- `refactor:` 代码重构
- `test:` 测试相关
- `chore:` 构建或辅助工具变动

---

## ❓ 常见问题

### Q1: 如何解决 "WebView2 not found" 错误？
**A**: Windows 系统需要安装 WebView2 Runtime：
- 访问 [Microsoft Edge WebView2](https://developer.microsoft.com/microsoft-edge/webview2/)
- 下载并安装 "Evergreen Bootstrapper"

### Q2: macOS 提示"无法打开应用程序"？
**A**: macOS 安全设置阻止了未签名应用：
```bash
# 允许运行
xattr -cr /Applications/Verdent账号管理器.app
```

### Q3: Linux 无法启动？
**A**: 安装必要的依赖：
```bash
# Ubuntu/Debian
sudo apt install libwebkit2gtk-4.1-0 libgtk-3-0

# Fedora
sudo dnf install webkit2gtk4.1 gtk3
```

### Q4: 自动注册失败？
**A**: 
- 检查网络连接
- 确保 Chrome 浏览器已安装
- 尝试减少并发数

### Q5: 数据存储在哪里？
**A**: 
- Windows: `%USERPROFILE%\.verdent_accounts\`
- macOS: `~/Library/Application Support/verdent_accounts/`
- Linux: `~/.verdent_accounts/`

---

## 📄 许可证

本项目采用 MIT 许可证 - 查看 [LICENSE](LICENSE) 文件了解详情

---

## 🙏 致谢

- [Tauri](https://tauri.app/) - 跨平台应用框架
- [Vue.js](https://vuejs.org/) - 前端框架
- [Rust](https://www.rust-lang.org/) - 系统编程语言
- [Playwright](https://playwright.dev/) - 自动化测试框架

---

## 📮 联系方式

- 项目主页: [https://github.com/chaogei/verdent-account-manager](https://github.com/chaogei/verdent-account-manager)
- Issues: [https://github.com/chaogei/verdent-account-manager/issues](https://github.com/chaogei/verdent-account-manager/issues)
- Email: chaogei666@gmail.com

---

<div align="center">
  <strong>⭐ 如果这个项目对你有帮助，请给一个 Star！</strong>
</div>
