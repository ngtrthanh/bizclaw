# Build Docker image locally for Windows
# This script prepares the binary and builds a single-platform Docker image

param(
    [string]$Tag = "bizclaw:local",
    [switch]$NoBuild
)

$ErrorActionPreference = "Stop"

Write-Host "🐳 Building Docker image locally" -ForegroundColor Cyan
Write-Host ""

# Check if Docker is running
try {
    docker version | Out-Null
} catch {
    Write-Host "❌ Docker is not running. Please start Docker Desktop." -ForegroundColor Red
    exit 1
}

# Build Rust binary if not skipped
if (-not $NoBuild) {
    Write-Host "🦀 Building Rust binary..." -ForegroundColor Yellow
    cargo build --release
    if ($LASTEXITCODE -ne 0) {
        Write-Host "❌ Cargo build failed" -ForegroundColor Red
        exit 1
    }
    Write-Host "✅ Binary built successfully" -ForegroundColor Green
    Write-Host ""
}

# Prepare docker-bin directory structure
Write-Host "📦 Preparing Docker build context..." -ForegroundColor Yellow

$Platform = "linux/amd64"
$BinDir = "docker-bin/$Platform"

# Create directory
New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

# For Windows, we'll use the Windows binary as a placeholder
# In production, you'd cross-compile or use the Linux binary
Write-Host "⚠️  Note: Using Windows binary as placeholder" -ForegroundColor Yellow
Write-Host "   For production, use cross-compiled Linux binary" -ForegroundColor Yellow

Copy-Item "target/release/bizclaw.exe" "$BinDir/bizclaw" -Force

Write-Host "✅ Build context prepared" -ForegroundColor Green
Write-Host ""

# Build Docker image
Write-Host "🐳 Building Docker image: $Tag" -ForegroundColor Yellow
Write-Host ""

docker build `
    --platform linux/amd64 `
    --build-arg TARGETPLATFORM=linux/amd64 `
    -t $Tag `
    .

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ Docker build failed" -ForegroundColor Red
    exit 1
}

Write-Host ""
Write-Host "✅ Docker image built successfully!" -ForegroundColor Green
Write-Host ""
Write-Host "📋 Image details:" -ForegroundColor Cyan
docker images $Tag

Write-Host ""
Write-Host "🚀 To run the container:" -ForegroundColor Cyan
Write-Host "   docker run --rm -it $Tag --help" -ForegroundColor White
Write-Host "   docker run --rm -it --init --cap-add=SYS_PTRACE $Tag serve" -ForegroundColor White
Write-Host ""
Write-Host "🐳 Or use docker-compose:" -ForegroundColor Cyan
Write-Host "   docker-compose up -d" -ForegroundColor White
