plugins {
    id("com.android.application")
    id("rust")
}

android {
    namespace="com.tessera.example"
    sourceSets.getByName("main").res.srcDir("src/main/tessera-res")
    compileSdk = 34
    defaultConfig {
        applicationId = "com.tessera.example"
        minSdk = 24
        targetSdk = 34
        versionCode = 1001
        versionName = "0.1.0"
    }
    compileOptions {
        sourceCompatibility = JavaVersion.VERSION_17
        targetCompatibility = JavaVersion.VERSION_17
    }
    sourceSets.getByName("main") {
        // Vulkan validation layers
        val ndkHome = System.getenv("NDK_HOME")
        jniLibs.srcDir("${ndkHome}/sources/third_party/vulkan/src/build-android/jniLibs")
    }
    buildTypes {
        getByName("debug") {
            isDebuggable = true
            isJniDebuggable = true
            isMinifyEnabled = false
            packaging {
                jniLibs.keepDebugSymbols.add("*/arm64-v8a/*.so")
                jniLibs.keepDebugSymbols.add("*/armeabi-v7a/*.so")
                jniLibs.keepDebugSymbols.add("*/x86/*.so")
                jniLibs.keepDebugSymbols.add("*/x86_64/*.so")
            }
        }
        getByName("release") {
            isMinifyEnabled = true
             proguardFiles(
                *fileTree(".") { include("**/*.pro") }
                    .plus(getDefaultProguardFile("proguard-android-optimize.txt"))
                    .toList().toTypedArray()
            )
        }
    }
}

rust {
    rootDirRel = "../../../"
    profilingOutput = System.getenv("TESSERA_PROFILING_OUTPUT")
    debugDirtyOverlay = System.getenv("TESSERA_DEBUG_DIRTY_OVERLAY") == "1"
}

dependencies {
    implementation("androidx.activity:activity:1.8.0")
    implementation("androidx.games:games-activity:4.4.0")
    implementation("androidx.appcompat:appcompat:1.6.1")
    implementation(project(":plugins:tessera_platform"))
}
