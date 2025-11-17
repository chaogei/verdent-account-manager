#!/bin/bash

# 准备资源文件脚本 (macOS/Linux)
# 确保 resources 目录包含所需的文件

echo -e "\033[32m准备资源文件...\033[0m"

# 创建 resources 目录
RESOURCES_DIR="Verdent_account_manger/resources"
if [ ! -d "$RESOURCES_DIR" ]; then
    mkdir -p "$RESOURCES_DIR"
    echo -e "\033[33m创建资源目录: $RESOURCES_DIR\033[0m"
fi

# 复制 Python 脚本到 resources 目录
SCRIPTS=(
    "verdent_auto_register.py"
    "verdent_auto_register_wrapper.py"
)

for SCRIPT in "${SCRIPTS[@]}"; do
    if [ -f "$SCRIPT" ]; then
        cp "$SCRIPT" "$RESOURCES_DIR/$SCRIPT"
        chmod +x "$RESOURCES_DIR/$SCRIPT"
        echo -e "\033[36m复制文件: $SCRIPT -> $RESOURCES_DIR/$SCRIPT\033[0m"
    else
        echo -e "\033[33m警告: 文件不存在 - $SCRIPT\033[0m"
    fi
done

echo -e "\n\033[32m资源准备完成!\033[0m"
echo -e "\033[32m现在可以运行 'npm run tauri dev' 或 'npm run tauri build'\033[0m"
