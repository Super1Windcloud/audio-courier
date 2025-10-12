#!/usr/bin/env bash
set -e

# 默认参数
Download=false
Check=false
Version="0.3.42"
ModelVersion="vosk-model-small-cn-0.22"
LargeModelVersion="vosk-model-cn-0.22"

# 参数解析
while [[ $# -gt 0 ]]; do
  case "$1" in
    -Download|--download)
      Download=true
      shift
      ;;
    -Check|--check)
      Check=true
      shift
      ;;
    -Version|--version)
      Version="$2"
      shift 2
      ;;
    -ModelVersion|--model-version)
      ModelVersion="$2"
      shift 2
      ;;
    -LargeModelVersion|--large-model-version)
      LargeModelVersion="$2"
      shift 2
      ;;
    *)
      echo "未知参数: $1"
      exit 1
      ;;
  esac
done

# 路径定义
VoskUrl="https://github.com/alphacep/vosk-api/releases/download/v0.3.42/vosk-osx-0.3.42.zip"
ModelUrl="https://alphacephei.com/vosk/models/${ModelVersion}.zip"
LargeModelUrl="https://alphacephei.com/vosk/models/${LargeModelVersion}.zip"

VoskDir="vosk-osx-${Version}"
ModelDir="${ModelVersion}"
ProjectRoot="$(pwd)"

echo -e "\033[1;32m=== Vosk 库设置脚本 ===\033[0m"

# === 检查模式 ===
if [ "$Check" = true ]; then
  echo -e "\033[1;33m检查 Vosk 库状态...\033[0m"

  voskPath="${ProjectRoot}/${VoskDir}"
  if [ -d "$voskPath" ]; then
    echo -e "\033[1;32m✓ Vosk 目录存在: $voskPath\033[0m"
    for dll in libvosk.dll libgcc_s_seh-1.dll libstdc++-6.dll libwinpthread-1.dll; do
      if [ -f "${voskPath}/${dll}" ]; then
        echo -e "\033[1;32m✓ 找到 DLL: ${dll}\033[0m"
      else
        echo -e "\033[1;31m✗ 缺少 DLL: ${dll}\033[0m"
      fi
    done
  else
    echo -e "\033[1;31m✗ Vosk 目录不存在: ${voskPath}\033[0m"
    echo -e "\033[1;33m请运行: ./setup_vosk.sh -Download\033[0m"
  fi

  modelPath="${ProjectRoot}/${ModelDir}"
  if [ -d "$modelPath" ]; then
    echo -e "\033[1;32m✓ 模型目录存在: $modelPath\033[0m"
  else
    echo -e "\033[1;31m✗ 模型目录不存在: $modelPath\033[0m"
    echo -e "\033[1;33m请运行: ./setup_vosk.sh -Download\033[0m"
  fi

  exit 0
fi

# === 下载模式 ===
if [ "$Download" = true ]; then
  echo -e "\033[1;33m下载 Vosk 库...\033[0m"

  voskZip="${ProjectRoot}/vosk-osx-${Version}.zip"
  modelZip="${ProjectRoot}/${ModelVersion}.zip"
  largeModelZip="${ProjectRoot}/${LargeModelVersion}.zip"

  set -x
  # 下载 Vosk 库
  echo -e "\033[1;36m从 $VoskUrl 下载 Vosk 库...\033[0m"
  curl -L -o "$voskZip" "$VoskUrl"
  echo -e "\033[1;32m✓ Vosk 下载完成\033[0m"

  echo -e "\033[1;36m解压 Vosk 文件...\033[0m"
  unzip -o "$voskZip" -d "$ProjectRoot"
  rm -f "$voskZip"
  echo -e "\033[1;32m✓ Vosk 解压完成\033[0m"

  # 下载模型
  echo -e "\033[1;36m从 $ModelUrl 下载中文模型 ($ModelVersion)...\033[0m"
  curl -L -o "$modelZip" "$ModelUrl"
  echo -e "\033[1;32m✓ 模型下载完成\033[0m"

  echo -e "\033[1;36m从 $LargeModelUrl 下载大型中文模型 ($LargeModelVersion)...\033[0m"
  curl -L -o "$largeModelZip" "$LargeModelUrl"
  echo -e "\033[1;32m✓ 模型下载完成\033[0m"

  echo -e "\033[1;36m解压模型...\033[0m"
  unzip -o "$modelZip" -d "$ProjectRoot"
  rm -f "$modelZip"
  unzip -o "$largeModelZip" -d "$ProjectRoot"
  rm -f "$largeModelZip"
  set +x

  echo -e "\033[1;32m✓ 所有资源准备完成！现在可以运行 Tauri 应用了。\033[0m"
  exit 0
fi

# === 默认帮助信息 ===
cat <<EOF
用法:
  ./setup_vosk.sh -Download            # 下载并设置 Vosk 库和中文模型
  ./setup_vosk.sh -Check               # 检查 Vosk 库与模型状态
  ./setup_vosk.sh -Download -Version "0.3.42" -ModelVersion "vosk-model-small-cn-0.22"

模型会被下载到:
  ${ProjectRoot}/${ModelVersion}
EOF
