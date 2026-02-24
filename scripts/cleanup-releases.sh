#!/bin/bash
# Cleanup old GitHub releases and artifacts
# Usage: ./scripts/cleanup-releases.sh [keep_count]

set -e

KEEP_COUNT=${1:-2}
REPO="ngtrthanh/bizclaw"

echo "🧹 Cleanup Script for GitHub Releases"
echo "Repository: $REPO"
echo "Keeping latest: $KEEP_COUNT releases"
echo ""

# Check if gh CLI is installed
if ! command -v gh &> /dev/null; then
    echo "❌ GitHub CLI (gh) is not installed"
    echo "Install it from: https://cli.github.com/"
    exit 1
fi

# Check if authenticated
if ! gh auth status &> /dev/null; then
    echo "❌ Not authenticated with GitHub CLI"
    echo "Run: gh auth login"
    exit 1
fi

echo "📋 Fetching releases..."
RELEASES=$(gh release list --repo $REPO --limit 100 --json tagName,createdAt --jq 'sort_by(.createdAt) | reverse | .[].tagName')

# Convert to array
RELEASE_ARRAY=($RELEASES)
TOTAL=${#RELEASE_ARRAY[@]}

echo "Found $TOTAL releases"
echo ""

if [ $TOTAL -le $KEEP_COUNT ]; then
    echo "✅ No releases to delete (total: $TOTAL, keeping: $KEEP_COUNT)"
    exit 0
fi

# Show what will be kept
echo "📌 Keeping these releases:"
for ((i=0; i<$KEEP_COUNT && i<$TOTAL; i++)); do
    echo "  - ${RELEASE_ARRAY[$i]}"
done
echo ""

# Show what will be deleted
DELETE_COUNT=$((TOTAL - KEEP_COUNT))
echo "🗑️  Will delete $DELETE_COUNT releases:"
for ((i=$KEEP_COUNT; i<$TOTAL; i++)); do
    echo "  - ${RELEASE_ARRAY[$i]}"
done
echo ""

# Confirm deletion
read -p "Continue with deletion? (y/N) " -n 1 -r
echo
if [[ ! $REPLY =~ ^[Yy]$ ]]; then
    echo "❌ Cancelled"
    exit 0
fi

# Delete old releases
echo ""
echo "🗑️  Deleting old releases..."
for ((i=$KEEP_COUNT; i<$TOTAL; i++)); do
    TAG="${RELEASE_ARRAY[$i]}"
    echo "  Deleting: $TAG"
    gh release delete "$TAG" --repo $REPO --yes --cleanup-tag || echo "  ⚠️  Failed to delete $TAG"
done

echo ""
echo "✅ Cleanup complete!"
echo ""

# Show storage usage
echo "📊 Current storage usage:"
gh api repos/$REPO --jq '.size' | awk '{printf "  Repository: %.2f MB\n", $1/1024}'

echo ""
echo "💡 To delete old artifacts and caches, run:"
echo "   gh workflow run cleanup-old-releases.yml --repo $REPO"
