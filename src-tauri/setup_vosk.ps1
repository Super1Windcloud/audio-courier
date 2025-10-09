param(
    [switch]$Download,
    [switch]$Check,
    [string]$Version = "0.3.45",
    [string]$ModelVersion = "vosk-model-small-cn-0.22"
)

$VoskUrl = "https://github.com/alphacep/vosk-api/releases/download/v$Version/vosk-win64-$Version.zip"
$ModelUrl = "https://alphacephei.com/vosk/models/$ModelVersion.zip"

$VoskDir = "vosk-win64-$Version"
$ModelDir = $ModelVersion
$ProjectRoot = $PSScriptRoot

Write-Host "=== Vosk 库设置脚本 ===" -ForegroundColor Green

if ($Check) {
    Write-Host "检查 Vosk 库状态..." -ForegroundColor Yellow

    $voskPath = Join-Path $ProjectRoot $VoskDir
    if (Test-Path $voskPath) {
        Write-Host "✓ Vosk 目录存在: $voskPath" -ForegroundColor Green
        $dllFiles = @("libvosk.dll", "libgcc_s_seh-1.dll", "libstdc++-6.dll", "libwinpthread-1.dll")
        foreach ($dll in $dllFiles) {
            $dllPath = Join-Path $voskPath $dll
            if (Test-Path $dllPath) {
                Write-Host "✓ 找到 DLL: $dll" -ForegroundColor Green
            } else {
                Write-Host "✗ 缺少 DLL: $dll" -ForegroundColor Red
            }
        }
    } else {
        Write-Host "✗ Vosk 目录不存在: $voskPath" -ForegroundColor Red
        Write-Host "请运行: .\setup_vosk.ps1 -Download" -ForegroundColor Yellow
    }

    $modelPath = Join-Path $ProjectRoot $ModelDir
    if (Test-Path $modelPath) {
        Write-Host "✓ 模型目录存在: $modelPath" -ForegroundColor Green
    } else {
        Write-Host "✗ 模型目录不存在: $modelPath" -ForegroundColor Red
        Write-Host "请运行: .\setup_vosk.ps1 -Download" -ForegroundColor Yellow
    }

    return
}

if ($Download) {
    Write-Host "下载 Vosk 库..." -ForegroundColor Yellow

    $voskZip = Join-Path $ProjectRoot "vosk-win64-$Version.zip"
    $modelZip = Join-Path $ProjectRoot "$ModelVersion.zip"

    try {
        # === 下载 Vosk 库 ===
        Write-Host "从 $VoskUrl 下载 Vosk 库..." -ForegroundColor Cyan
        Invoke-WebRequest -Uri $VoskUrl -OutFile $voskZip -UseBasicParsing
        Write-Host "✓ Vosk 下载完成" -ForegroundColor Green

        Write-Host "解压 Vosk 文件..." -ForegroundColor Cyan
        Expand-Archive -Path $voskZip -DestinationPath $ProjectRoot -Force
        Write-Host "✓ Vosk 解压完成" -ForegroundColor Green
        Remove-Item $voskZip -Force

        # === 下载模型 ===
        Write-Host "从 $ModelUrl 下载中文模型 ($ModelVersion) ..." -ForegroundColor Cyan
        Invoke-WebRequest -Uri $ModelUrl -OutFile $modelZip -UseBasicParsing
        Write-Host "✓ 模型下载完成" -ForegroundColor Green

        Write-Host "解压模型..." -ForegroundColor Cyan
        Expand-Archive -Path $modelZip -DestinationPath $ProjectRoot -Force
        Write-Host "✓ 模型解压完成" -ForegroundColor Green
        Remove-Item $modelZip -Force

        Write-Host "✓ 所有资源准备完成！现在可以运行 Tauri 应用了。" -ForegroundColor Green
    }
    catch {
        Write-Host "✗ 下载或解压失败: $($_.Exception.Message)" -ForegroundColor Red
        Write-Host "请手动下载并解压 Vosk 库或模型。" -ForegroundColor Yellow
    }
    return
}

# 默认帮助信息
Write-Host @"
用法:
  .\setup_vosk.ps1 -Download            # 下载并设置 Vosk 库和中文模型
  .\setup_vosk.ps1 -Check               # 检查 Vosk 库与模型状态
  .\setup_vosk.ps1 -Download -Version "0.3.45" -ModelVersion "vosk-model-small-cn-0.22"

模型会被下载到:
  $ProjectRoot\$ModelVersion
"@ -ForegroundColor Cyan
