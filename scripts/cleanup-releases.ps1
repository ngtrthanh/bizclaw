# Cleanup old GitHub releases and artifacts
# Usage: .\scripts\cleanup-releases.ps1 [keep_count]

param(
    [int]$KeepCount = 2
)

$ErrorActionPreference = "Stop"
$Repo = "ngtrthanh/bizclaw"

Write-Host "🧹 Cleanup Script for GitHub Releases" -ForegroundColor Cyan
Write-Host "Repository: $Repo"
Write-Host "Keeping latest: $KeepCount releases"
Write-Host ""

# Check if gh CLI is installed
if (!(Get-Command gh -ErrorAction SilentlyContinue)) {
    Write-Host "❌ GitHub CLI (gh) is not installed" -ForegroundColor Red
    Write-Host "Install it from: https://cli.github.com/"
    exit 1
}

# Check if authenticated
try {
    gh auth status 2>&1 | Out-Null
} catch {
    Write-Host "❌ Not authenticated with GitHub CLI" -ForegroundColor Red
    Write-Host "Run: gh auth login"
    exit 1
}

Write-Host "📋 Fetching releases..."
$ReleasesJson = gh release list --repo $Repo --limit 100 --json tagName,createdAt | ConvertFrom-Json
$Releases = $ReleasesJson | Sort-Object -Property createdAt -Descending | Select-Object -ExpandProperty tagName

$Total = $Releases.Count
Write-Host "Found $Total releases"
Write-Host ""

if ($Total -le $KeepCount) {
    Write-Host "✅ No releases to delete (total: $Total, keeping: $KeepCount)" -ForegroundColor Green
    exit 0
}

# Show what will be kept
Write-Host "📌 Keeping these releases:" -ForegroundColor Green
for ($i = 0; $i -lt $KeepCount -and $i -lt $Total; $i++) {
    Write-Host "  - $($Releases[$i])"
}
Write-Host ""

# Show what will be deleted
$DeleteCount = $Total - $KeepCount
Write-Host "🗑️  Will delete $DeleteCount releases:" -ForegroundColor Yellow
for ($i = $KeepCount; $i -lt $Total; $i++) {
    Write-Host "  - $($Releases[$i])"
}
Write-Host ""

# Confirm deletion
$Confirmation = Read-Host "Continue with deletion? (y/N)"
if ($Confirmation -ne 'y' -and $Confirmation -ne 'Y') {
    Write-Host "❌ Cancelled" -ForegroundColor Red
    exit 0
}

# Delete old releases
Write-Host ""
Write-Host "🗑️  Deleting old releases..." -ForegroundColor Yellow
for ($i = $KeepCount; $i -lt $Total; $i++) {
    $Tag = $Releases[$i]
    Write-Host "  Deleting: $Tag"
    try {
        gh release delete $Tag --repo $Repo --yes --cleanup-tag
    } catch {
        Write-Host "  ⚠️  Failed to delete $Tag" -ForegroundColor Yellow
    }
}

Write-Host ""
Write-Host "✅ Cleanup complete!" -ForegroundColor Green
Write-Host ""

# Show storage usage
Write-Host "📊 Current storage usage:" -ForegroundColor Cyan
$RepoInfo = gh api repos/$Repo | ConvertFrom-Json
$SizeMB = [math]::Round($RepoInfo.size / 1024, 2)
Write-Host "  Repository: $SizeMB MB"

Write-Host ""
Write-Host "💡 To delete old artifacts and caches, run:" -ForegroundColor Cyan
Write-Host "   gh workflow run cleanup-old-releases.yml --repo $Repo"
