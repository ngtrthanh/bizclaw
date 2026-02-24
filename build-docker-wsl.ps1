# Build Docker image using WSL for proper Linux binary
# This ensures the binary is actually a Linux binary, not Windows

param(
    [string]$Tag = "bizclaw:local"
)

$ErrorActionPreference = "Stop"

Write-Host "🐳 Building Docker image using WSL" -ForegroundColor Cyan
Write-Host ""

# Check if WSL is available
try {
    wsl --version | Out-Null
} catch {
    Write-Host "❌ WSL is not available. Install WSL or use build-docker-local.ps1" -ForegroundColor Red
    Write-Host "   Install WSL: wsl --install" -ForegroundColor Yellow
    exit 1
}

# Check if Docker is running
try {
    docker version | Out-Null
} catch {
    Write-Host "❌ Docker is not running. Please start Docker Desktop." -ForegroundColor Red
    exit 1
}

Write-Host "🦀 Building Linux binary in WSL..." -ForegroundColor Yellow
Write-Host ""

# Build in WSL
wsl bash -c @"
cd /mnt/c/Users/thanh/Downloads/26022-dev/bizclaw
cargo build --release --target x86_64-unknown-linux-gnu
"@

if ($LASTEXITCODE -ne 0) {
    Write-Host "❌ WSL build failed" -ForegroundColor Red
    Write-Host "   Make sure Rust is installed in WSL: curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh" -ForegroundColor Yellow
    exit 1
}

Write-Host "✅ Linux binary built" -ForegroundColor Green
Write-Host ""

# Prepare docker-bin directory
Write-Host "📦 Preparing Docker build context..." -ForegroundColor Yellow

$Platform = "linux/amd64"
$BinDir = "docker-bin/$Platform"

New-Item -ItemType Directory -Force -Path $BinDir | Out-Null

# Copy Linux binary
Copy-Item "target/x86_64-unknown-linux-gnu/release/bizclaw" "$BinDir/bizclaw" -Force

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
Write-Host "   docker run --rm -it --init --cap-add=SYS_PTRACE $Tag --help" -ForegroundColor White
Write-Host ""
Write-Host "🐳 Or use docker-compose:" -ForegroundColor Cyan
Write-Host "   Update docker-compose.yml to use 'image: $Tag'" -ForegroundColor White
Write-Host "   docker-compose up -d" -ForegroundColor White
