#!/bin/bash

# GitHub Repository Setup Script for Sparka
# Run these commands to create repository and push code

echo "=== Sparka GitHub Setup ==="
echo ""

# 1. Create GitHub repository (you need to do this manually or use GitHub CLI)
echo "1. Create a new repository on GitHub:"
echo "   - Go to https://github.com/new"
echo "   - Repository name: sparka"
echo "   - Description: Android ticket manager with Google Drive integration"
echo "   - Make it Public or Private as you prefer"
echo "   - Don't initialize with README (we already have one)"
echo ""

# 2. Add remote and push
echo "2. Add remote and push to GitHub:"
echo "   git remote add origin https://github.com/YOUR_USERNAME/sparka.git"
echo "   git branch -M main"
echo "   git push -u origin main"
echo ""

# 3. Create release to trigger build
echo "3. Create a release to trigger the GitHub Actions build:"
echo "   - Go to https://github.com/YOUR_USERNAME/sparka/releases/new"
echo "   - Tag version: v1.0.0"
echo "   - Release title: Initial Release"
echo "   - Description: First release of Sparka Android app"
echo "   - Click 'Publish release'"
echo ""

# 4. Monitor build
echo "4. Monitor the build:"
echo "   - Go to https://github.com/YOUR_USERNAME/sparka/actions"
echo "   - Watch the Android build workflow run"
echo "   - APK will be uploaded to the release when complete"
echo ""

echo "=== Configuration Needed ==="
echo "Before building, you'll need to:"
echo "1. Set up Google Cloud Console OAuth credentials"
echo "2. Get Groq API key"
echo "3. Update placeholder values in the code:"
echo "   - YOUR_CLIENT_ID (in MainActivity.java)"
echo "   - YOUR_WEB_CLIENT_ID (in MainActivity.java)"
echo "   - YOUR_GROQ_API_KEY (in groq.rs)"
echo ""

echo "=== GitHub Actions Workflow ==="
echo "The workflow will:"
echo "1. Install Rust and cargo-mobile2"
echo "2. Initialize Android project"
echo "3. Build APK"
echo "4. Upload to GitHub releases"
echo "5. Cache dependencies for faster builds"
echo ""

echo "Ready to push to GitHub! 🚀"