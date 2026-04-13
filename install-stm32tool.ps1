# ------------------------------------------------------------
# stm32tool Windows Installer (English Version)
# ------------------------------------------------------------

Add-Type -AssemblyName System.Windows.Forms

# ------------------------------------------------------------
# Helper: File Dialog for STM32CubeMX.exe
# ------------------------------------------------------------
function Select-CubeMXPath {
    $dialog = New-Object System.Windows.Forms.OpenFileDialog
    $dialog.Title  = "Configuration: Select STM32CubeMX.exe"
    $dialog.Filter = "STM32CubeMX Executable (STM32CubeMX.exe)|STM32CubeMX.exe"
    $dialog.Multiselect = $false

    $result = $dialog.ShowDialog()
    if ($result -ne [System.Windows.Forms.DialogResult]::OK) {
        Write-Host "Error: Installation aborted because STM32CubeMX.exe was not selected." -ForegroundColor Red
        exit 1
    }

    # Return the folder containing the .exe
    return (Split-Path -Parent $dialog.FileName)
}

# ------------------------------------------------------------
# Step 1: Environment Variable Check (STM32CubeMX_PATH)
# ------------------------------------------------------------
Write-Host ">>> Step 1: Checking environment variables..." -ForegroundColor Cyan
$cubemxDir = [Environment]::GetEnvironmentVariable("STM32CubeMX_PATH", "User")

$validCubeMX = $false
if ($cubemxDir -and $cubemxDir.Trim().Length -gt 0) {
    if (Test-Path (Join-Path $cubemxDir "STM32CubeMX.exe")) {
        $validCubeMX = $true
    }
}

if (-not $validCubeMX) {
    Write-Host "[!] STM32CubeMX_PATH not found or invalid. Please select it manually." -ForegroundColor Yellow
    $cubemxDir = Select-CubeMXPath

    [Environment]::SetEnvironmentVariable(
        "STM32CubeMX_PATH",
        $cubemxDir,
        "User"
    )
    Write-Host "[+] Successfully set STM32CubeMX_PATH to: $cubemxDir" -ForegroundColor Green
} else {
    Write-Host "[+] Detected STM32CubeMX at: $cubemxDir" -ForegroundColor Green
}

# ------------------------------------------------------------
# Step 2: Fetch Latest Release Info from GitHub
# ------------------------------------------------------------
Write-Host "`n>>> Step 2: Fetching latest release info from GitHub..." -ForegroundColor Cyan
$apiUrl = "https://api.github.com/repos/HITSZ-WTRobot/stm32tool/releases/latest"
$headers = @{
    "User-Agent" = "PowerShell"
    "Accept"     = "application/vnd.github+json"
}

try {
    $release = Invoke-RestMethod -Uri $apiUrl -Headers $headers -ErrorAction Stop
} catch {
    Write-Host "Error: Failed to connect to GitHub API. Please check your internet connection." -ForegroundColor Red
    exit 1
}

$version = $release.tag_name
$assetName = "stm32tool-windows-$version.exe"
$asset = $release.assets | Where-Object { $_.name -eq $assetName }

if (-not $asset) {
    Write-Host "Error: Could not find the required asset: $assetName" -ForegroundColor Red
    exit 1
}

# ------------------------------------------------------------
# Step 3: Download with Progress Bar
# ------------------------------------------------------------
$installDir = Join-Path $env:LOCALAPPDATA "stm32tool"
$exePath    = Join-Path $installDir "stm32tool.exe"

if (-not (Test-Path $installDir)) {
    New-Item -ItemType Directory -Path $installDir | Out-Null
}

Write-Host ">>> Step 3: Downloading $assetName..." -ForegroundColor Cyan

# Start download with Progress Bar
try {
    Invoke-WebRequest -Uri $asset.browser_download_url -OutFile $exePath -UserAgent "PowerShell"
} catch {
    Write-Host "Error: Download failed." -ForegroundColor Red
    exit 1
}

# ------------------------------------------------------------
# Step 4: Update User PATH
# ------------------------------------------------------------
Write-Host "`n>>> Step 4: Updating system PATH..." -ForegroundColor Cyan
$currentPath = [Environment]::GetEnvironmentVariable("Path", "User")

$pathList = @()
if ($currentPath) {
    $pathList = $currentPath -split ";"
}

# Clean up trailing slashes for comparison
$normalizedInstallDir = $installDir.TrimEnd('\')
$normalizedPathList = $pathList | ForEach-Object { $_.TrimEnd('\') }

if ($normalizedPathList -notcontains $normalizedInstallDir) {
    $newPath = ($pathList + $installDir) -join ";"
    [Environment]::SetEnvironmentVariable("Path", $newPath, "User")
    Write-Host "[+] Installation directory added to User PATH." -ForegroundColor Green
} else {
    Write-Host "[*] PATH already contains the installation directory." -ForegroundColor Gray
}

# ------------------------------------------------------------
# Final Message
# ------------------------------------------------------------
Write-Host "`n====================================================" -ForegroundColor Magenta
Write-Host " SUCCESS: stm32tool $version installed!" -ForegroundColor Green
Write-Host " Please RESTART your terminal/IDE to apply changes." -ForegroundColor Yellow
Write-Host "====================================================`n"
