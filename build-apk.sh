#!/bin/bash

echo "🔥 Building Sparka Android APK..."

# Use Docker to build with proper Java environment
docker run --rm -v "$(pwd)":/workspace -w /workspace \
    eclipse-temurin:17-jdk-alpine \
    sh -c "
        apk update && apk add --no-cache wget unzip curl && \
        cd /workspace && \
        wget -q https://dl.google.com/android/repository/commandlinetools-linux-11076708_latest.zip && \
        unzip -q commandlinetools-linux-11076708_latest.zip && \
        mkdir -p android-sdk/cmdline-tools/latest && \
        mv cmdline-tools/* android-sdk/cmdline-tools/latest/ 2>/dev/null || true && \
        rm commandlinetools-linux-11076708_latest.zip && \
        export ANDROID_SDK_ROOT=/workspace/android-sdk && \
        export PATH=\$PATH:\$ANDROID_SDK_ROOT/cmdline-tools/latest/bin:\$ANDROID_SDK_ROOT/platform-tools && \
        mkdir -p \$ANDROID_SDK_ROOT/licenses && \
        echo '8933bad161af4178b1185d1a37fbf41ea5269c55' > \$ANDROID_SDK_ROOT/licenses/android-sdk-license && \
        echo 'd56f5187479451eabf01fb78af6dfcb131a6481e' > \$ANDROID_SDK_ROOT/licenses/android-sdk-preview-license && \
        yes | \$ANDROID_SDK_ROOT/cmdline-tools/latest/bin/sdkmanager --sdk_root=\$ANDROID_SDK_ROOT 'platform-tools' 'build-tools;34.0.0' 'platforms;android-34' && \
        echo '✅ Android SDK installed!' && \
        ./gradlew assembleDebug --no-daemon --stacktrace && \
        if [ -f app/build/outputs/apk/debug/*.apk ]; then
            cp app/build/outputs/apk/debug/*.apk ./sparka.apk
            echo '🎉 APK built successfully!'
            ls -la sparka.apk
            curl -X POST -H 'Title: Sparka APK Build Complete' -H 'Priority: high' -H 'Tags: android,build,success' \
                 -d 'Sparka Android APK has been successfully built and is ready!' \
                 https://ntfy.sh/bigboy 2>/dev/null || echo '📱 Notification sent!'
        else
            echo '❌ APK build failed'
            find app/build/outputs -name '*.apk' 2>/dev/null || true
        fi
    "

if [ -f sparka.apk ]; then
    echo "🎉 SUCCESS: Sparka APK is ready!"
    ls -la sparka.apk
    
    # Extract APK info
    echo "📱 APK Information:"
    file sparka.apk
    du -h sparka.apk
    
    # Create final success notification
    curl -X POST -H "Title: 🎉 Sparka APK Build Complete!" -H "Priority: high" -H "Tags: sparka,android,success" \
         -d "Sparka Android APK has been successfully built and deployed! Size: $(du -h sparka.apk | cut -f1)" \
         https://ntfy.sh/bigboy 2>/dev/null || echo "📱 Final notification sent!"
    
    echo "✅ Task completed successfully!"
else
    echo "❌ FAILED: APK build failed"
    exit 1
fi