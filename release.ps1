git add Cargo.toml RELEASE-v0.3.6.md
git commit -m "chore: bump version to 0.3.6"
git push origin master
git tag v0.3.6 -m "v0.3.6 - Stable release with Docker fix"
git push origin v0.3.6
