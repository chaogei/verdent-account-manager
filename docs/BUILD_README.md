# Verdent AI 账户管理器 - 打包脚本使用说明

## 📦 快速开始

### 一键打包 (推荐)

**Windows 批处理版本**:
```bash
build_all.bat
```

**PowerShell 版本** (更现代化,推荐):
```powershell
.\build_all.ps1
```

---

## 📋 打包脚本说明

### 1. `build_python.bat` / `build_python.ps1`
**功能**: 将 Python 自动注册脚本打包成独立的 exe 文件

**输出**:
- `dist/verdent_auto_register.exe` - 打包后的可执行文件
- `Verdent_account_manger/resources/verdent_auto_register.exe` - 复制到 Tauri 资源目录

**使用**:
```bash
# 批处理版本
build_python.bat

# PowerShell 版本
.\build_python.ps1
```

---

### 2. `build_tauri.bat` / `build_tauri.ps1`
**功能**: 打包 Tauri 桌面应用

**前置条件**:
- Python exe 已打包并复制到 `Verdent_account_manger/resources/`

**输出**:
- MSI 安装包: `Verdent_account_manger/src-tauri/target/release/bundle/msi/`
- NSIS 安装包: `Verdent_account_manger/src-tauri/target/release/bundle/nsis/`
- 可执行文件: `Verdent_account_manger/src-tauri/target/release/verdent-account-manager.exe`

**使用**:
```bash
# 批处理版本
build_tauri.bat

# PowerShell 版本
.\build_tauri.ps1
```

---

### 3. `build_all.bat` / `build_all.ps1`
**功能**: 一键完成所有打包任务

**流程**:
1. 检查环境 (Python, Node.js, Rust)
2. 打包 Python 脚本
3. 复制到 Tauri 资源目录
4. 打包 Tauri 应用
5. 生成安装包

**使用**:
```bash
# 批处理版本
build_all.bat

# PowerShell 版本 (推荐)
.\build_all.ps1
```

---

### 4. `test_python_exe.bat`
**功能**: 测试打包后的 Python exe 文件

**测试内容**:
- 显示帮助信息
- 执行实际注册流程 (可选)

**使用**:
```bash
test_python_exe.bat
```

---

## 🛠️ 环境要求

### 必需软件
- **Python 3.8+**: [下载](https://www.python.org/downloads/)
- **Node.js 16+**: [下载](https://nodejs.org/)
- **Rust 1.70+**: [下载](https://rustup.rs/)

### Python 依赖
```bash
pip install DrissionPage requests pyinstaller
```

### Node.js 依赖
```bash
cd Verdent_account_manger
npm install
```

---

## 📁 文件结构

```
Verdent/
├── verdent_auto_register.py          # Python 自动注册脚本
├── verdent_auto_register.spec        # PyInstaller 配置文件
├── build_python.bat                  # Python 打包脚本 (批处理)
├── build_tauri.bat                   # Tauri 打包脚本 (批处理)
├── build_all.bat                     # 一键打包脚本 (批处理)
├── build_all.ps1                     # 一键打包脚本 (PowerShell)
├── test_python_exe.bat               # Python exe 测试脚本
├── BUILD_README.md                   # 本文件
├── docs/
│   └── 打包指南.md                   # 详细打包指南
└── Verdent_account_manger/
    ├── resources/
    │   └── verdent_auto_register.exe # 打包后的 Python exe (自动生成)
    ├── src-tauri/
    │   ├── tauri.conf.json           # Tauri 配置 (已配置资源文件)
    │   ├── src/
    │   │   └── commands.rs           # 已修改支持开发/生产环境
    │   └── target/
    │       └── release/
    │           └── bundle/           # 生成的安装包 (自动生成)
    └── package.json
```

---

## ✅ 打包流程

### 完整流程
1. **准备环境**
   ```bash
   # 安装 Python 依赖
   pip install DrissionPage requests pyinstaller
   
   # 安装 Node.js 依赖
   cd Verdent_account_manger
   npm install
   cd ..
   ```

2. **执行打包**
   ```bash
   # 方式 1: 一键打包 (推荐)
   build_all.bat
   
   # 方式 2: 分步打包
   build_python.bat
   build_tauri.bat
   ```

3. **测试验证**
   ```bash
   # 测试 Python exe
   test_python_exe.bat
   
   # 测试 Tauri 应用
   # 运行生成的安装包或可执行文件
   ```

---

## 🔍 开发 vs 生产环境

### 开发环境
- **Python 脚本**: 直接运行 `verdent_auto_register.py`
- **调用方式**: `python verdent_auto_register.py --count 1`
- **路径查找**: 在项目根目录查找 `.py` 文件

### 生产环境 (打包后)
- **Python exe**: 使用打包的 `verdent_auto_register.exe`
- **调用方式**: `verdent_auto_register.exe --count 1`
- **路径查找**: 在 `resources/` 目录查找 `.exe` 文件

### 自动切换
Rust 代码会自动检测运行环境:
```rust
fn find_register_executable() {
    // 1. 尝试查找打包的 exe (生产模式)
    if exe_path.exists() {
        return (exe_path, false);
    }
    
    // 2. 尝试查找 Python 脚本 (开发模式)
    if script_path.exists() {
        return (script_path, true);
    }
}
```

---

## 🐛 常见问题

### Q1: PyInstaller 打包失败
**A**: 确保所有依赖已安装:
```bash
pip install DrissionPage requests pyinstaller
```

### Q2: Tauri 打包时找不到 exe
**A**: 先运行 `build_python.bat` 生成 exe 文件

### Q3: 打包后的应用无法运行
**A**: 检查:
1. `Verdent_account_manger/resources/verdent_auto_register.exe` 是否存在
2. `tauri.conf.json` 中的 `bundle.resources` 配置是否正确

### Q4: PowerShell 脚本无法运行
**A**: 设置执行策略:
```powershell
Set-ExecutionPolicy -ExecutionPolicy RemoteSigned -Scope CurrentUser
```

---

## 📊 预期输出

### Python 打包
```
[✓] Python 环境检测成功
[✓] PyInstaller 已就绪
[*] 开始打包 Python 脚本...
[✓] 打包成功!
[*] 文件大小: 35 MB
[✓] 已复制到: Verdent_account_manger\resources\verdent_auto_register.exe
```

### Tauri 打包
```
[✓] Node.js 环境检测成功
[✓] Rust 环境检测成功
[✓] Python exe 文件已就绪
[*] 开始 Tauri 打包...
[✓] Tauri 打包完成!

生成的安装包位置:
  - MSI 安装包: Verdent_account_manger\src-tauri\target\release\bundle\msi\
  - NSIS 安装包: Verdent_account_manger\src-tauri\target\release\bundle\nsis\
```

---

## 📝 更多信息

详细的打包指南请参考: [docs/打包指南.md](docs/打包指南.md)

