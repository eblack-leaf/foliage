//! Every text file here is a parameterized copy of `rust-mobile/android-activity`'s own
//! `agdk-mainloop` example project (the crate that implements winit's `android-game-activity`
//! backend) -- verified working against `androidx.games:games-activity`, not invented from
//! scratch. Only the values that have to vary per-app (application id, cdylib name, SDK
//! levels) are parameters; everything else is copied as-is.

pub fn root_build_gradle(agp_version: &str) -> String {
    let mut out = String::new();
    out.push_str("// Top-level build file where you can add configuration options common to all sub-projects/modules.\n");
    out.push_str("plugins {\n");
    out.push_str(&format!(
        "    id 'com.android.application' version '{agp_version}' apply false\n"
    ));
    out.push_str(&format!(
        "    id 'com.android.library' version '{agp_version}' apply false\n"
    ));
    out.push_str("}\n");
    out
}

pub fn settings_gradle() -> String {
    let mut out = String::new();
    out.push_str("pluginManagement {\n");
    out.push_str("    repositories {\n");
    out.push_str("        gradlePluginPortal()\n");
    out.push_str("        google()\n");
    out.push_str("        mavenCentral()\n");
    out.push_str("    }\n");
    out.push_str("}\n");
    out.push_str("dependencyResolutionManagement {\n");
    out.push_str("    repositoriesMode.set(RepositoriesMode.FAIL_ON_PROJECT_REPOS)\n");
    out.push_str("    repositories {\n");
    out.push_str("        google()\n");
    out.push_str("        mavenCentral()\n");
    out.push_str("    }\n");
    out.push_str("}\n");
    out.push_str("include ':app'\n");
    out
}

pub fn gradle_properties() -> String {
    let mut out = String::new();
    out.push_str("# Enable Gradle Daemon\n");
    out.push_str("org.gradle.daemon=true\n");
    out.push_str("# JVM arguments\n");
    out.push_str(
        "org.gradle.jvmargs=-Xmx4g -XX:+HeapDumpOnOutOfMemoryError -Dfile.encoding=UTF-8\n",
    );
    out.push_str("# Enable AndroidX\n");
    out.push_str("android.useAndroidX=true\n");
    out.push_str("# Build caching and parallel execution\n");
    out.push_str("org.gradle.caching=true\n");
    out.push_str("org.gradle.parallel=true\n");
    out.push_str("# File system watching for faster builds\n");
    out.push_str("org.gradle.unsafe.watch-fs=true\n");
    out
}

pub fn gradle_wrapper_properties(gradle_version: &str) -> String {
    let mut out = String::new();
    out.push_str("distributionBase=GRADLE_USER_HOME\n");
    out.push_str("distributionPath=wrapper/dists\n");
    out.push_str(&format!(
        "distributionUrl=https\\://services.gradle.org/distributions/gradle-{gradle_version}-bin.zip\n"
    ));
    out.push_str("networkTimeout=10000\n");
    out.push_str("validateDistributionUrl=true\n");
    out.push_str("zipStoreBase=GRADLE_USER_HOME\n");
    out.push_str("zipStorePath=wrapper/dists\n");
    out
}

pub fn app_build_gradle(
    app_id: &str,
    min_sdk: u32,
    compile_sdk: u32,
    target_sdk: u32,
    games_activity_version: &str,
) -> String {
    let mut out = String::new();
    out.push_str("plugins {\n");
    out.push_str("    id 'com.android.application'\n");
    out.push_str("}\n\n");
    out.push_str(
        "// Release signing: optional, and absent by default -- `assembleRelease` still\n",
    );
    out.push_str(
        "// works without it (produces an unsigned artifact you can't install directly).\n",
    );
    out.push_str(
        "// Create keystore.properties next to this file (see android/README.md's \"Release\n",
    );
    out.push_str(
        "// signing\" section) and it's picked up automatically -- nothing else to edit here.\n",
    );
    out.push_str("def keystorePropertiesFile = rootProject.file(\"keystore.properties\")\n");
    out.push_str("def keystoreProperties = new Properties()\n");
    out.push_str("if (keystorePropertiesFile.exists()) {\n");
    out.push_str("    keystoreProperties.load(new FileInputStream(keystorePropertiesFile))\n");
    out.push_str("}\n\n");
    out.push_str("android {\n");
    out.push_str(&format!("    compileSdk = {compile_sdk}\n\n"));
    out.push_str("    defaultConfig {\n");
    out.push_str(&format!("        applicationId = \"{app_id}\"\n"));
    out.push_str(&format!("        minSdk = {min_sdk}\n"));
    out.push_str(&format!("        targetSdk = {target_sdk}\n"));
    out.push_str("        versionCode = 1\n");
    out.push_str("        versionName = \"1.0\"\n");
    out.push_str("    }\n\n");
    out.push_str("    signingConfigs {\n");
    out.push_str("        release {\n");
    out.push_str("            if (keystorePropertiesFile.exists()) {\n");
    out.push_str("                storeFile rootProject.file(keystoreProperties['storeFile'])\n");
    out.push_str("                storePassword keystoreProperties['storePassword']\n");
    out.push_str("                keyAlias keystoreProperties['keyAlias']\n");
    out.push_str("                keyPassword keystoreProperties['keyPassword']\n");
    out.push_str("            }\n");
    out.push_str("        }\n");
    out.push_str("    }\n\n");
    out.push_str("    buildTypes {\n");
    out.push_str("        release {\n");
    out.push_str("            minifyEnabled = false\n");
    out.push_str("            if (keystorePropertiesFile.exists()) {\n");
    out.push_str("                signingConfig signingConfigs.release\n");
    out.push_str("            }\n");
    out.push_str("        }\n");
    out.push_str("        debug {\n");
    out.push_str("            minifyEnabled = false\n");
    out.push_str("        }\n");
    out.push_str("    }\n");
    out.push_str("    compileOptions {\n");
    out.push_str("        sourceCompatibility JavaVersion.VERSION_17\n");
    out.push_str("        targetCompatibility JavaVersion.VERSION_17\n");
    out.push_str("    }\n");
    out.push_str(&format!("    namespace = '{app_id}'\n"));
    out.push_str("}\n\n");
    out.push_str("dependencies {\n");
    out.push_str("    implementation 'androidx.appcompat:appcompat:1.7.0'\n\n");
    out.push_str("    // To use the Games Activity library\n");
    out.push_str(&format!(
        "    implementation \"androidx.games:games-activity:{games_activity_version}\"\n"
    ));
    out.push_str(
        "    // Note: don't include game-text-input separately, since it's integrated into game-activity\n",
    );
    out.push_str("}\n");
    out
}

pub fn android_manifest(app_name: &str, lib_name: &str) -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    out.push_str("<manifest xmlns:android=\"http://schemas.android.com/apk/res/android\">\n\n");
    out.push_str("    <application\n");
    out.push_str("        android:icon=\"@android:drawable/sym_def_app_icon\"\n");
    out.push_str(&format!("        android:label=\"{app_name}\"\n"));
    out.push_str("        android:supportsRtl=\"true\"\n");
    out.push_str("        android:theme=\"@style/ActivityTheme\">\n");
    out.push_str("        <activity\n");
    out.push_str("            android:name=\".MainActivity\"\n");
    out.push_str(
        "            android:configChanges=\"orientation|screenSize|screenLayout|keyboardHidden\"\n",
    );
    out.push_str("            android:exported=\"true\">\n");
    out.push_str("            <intent-filter>\n");
    out.push_str("                <action android:name=\"android.intent.action.MAIN\" />\n");
    out.push_str(
        "                <category android:name=\"android.intent.category.LAUNCHER\" />\n",
    );
    out.push_str("            </intent-filter>\n\n");
    out.push_str(&format!(
        "            <meta-data android:name=\"android.app.lib_name\" android:value=\"{lib_name}\" />\n"
    ));
    out.push_str("        </activity>\n");
    out.push_str("    </application>\n\n");
    out.push_str("</manifest>\n");
    out
}

pub fn themes_xml() -> String {
    let mut out = String::new();
    out.push_str("<?xml version=\"1.0\" encoding=\"utf-8\"?>\n");
    out.push_str("<resources>\n");
    out.push_str(
        "    <style name=\"ActivityTheme\" parent=\"Theme.AppCompat.Light.NoActionBar\">\n",
    );
    out.push_str("        <!-- For full-screen layout, cutout support -->\n");
    out.push_str(
        "        <item name=\"android:windowLayoutInDisplayCutoutMode\">shortEdges</item>\n",
    );
    out.push_str("        <item name=\"android:windowFullscreen\">true</item>\n");
    out.push_str("    </style>\n");
    out.push_str("</resources>\n");
    out
}

pub fn main_activity_java(app_id: &str, lib_name: &str) -> String {
    let mut out = String::new();
    out.push_str(&format!("package {app_id};\n\n"));
    out.push_str("import androidx.core.view.WindowCompat;\n");
    out.push_str("import androidx.core.view.WindowInsetsCompat;\n");
    out.push_str("import androidx.core.view.WindowInsetsControllerCompat;\n\n");
    out.push_str("import com.google.androidgamesdk.GameActivity;\n\n");
    out.push_str("import android.os.Bundle;\n");
    out.push_str("import android.view.View;\n");
    out.push_str("import android.view.WindowManager;\n\n");
    out.push_str("public class MainActivity extends GameActivity {\n\n");
    out.push_str("    static {\n");
    out.push_str(
        "        // Must match both the foliage app's `[lib] name` (its cdylib output) and\n",
    );
    out.push_str(
        "        // the `android.app.lib_name` meta-data value in AndroidManifest.xml -- all\n",
    );
    out.push_str(
        "        // three have to agree, or the JVM can't find `android_main` to call into.\n",
    );
    out.push_str(&format!("        System.loadLibrary(\"{lib_name}\");\n"));
    out.push_str("    }\n\n");
    out.push_str("    private void hideSystemUI() {\n");
    out.push_str("        getWindow().getAttributes().layoutInDisplayCutoutMode\n");
    out.push_str(
        "                = WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_ALWAYS;\n",
    );
    out.push_str("        View decorView = getWindow().getDecorView();\n");
    out.push_str(
        "        WindowInsetsControllerCompat controller = new WindowInsetsControllerCompat(getWindow(),\n",
    );
    out.push_str("                decorView);\n");
    out.push_str("        controller.hide(WindowInsetsCompat.Type.systemBars());\n");
    out.push_str("        controller.hide(WindowInsetsCompat.Type.displayCutout());\n");
    out.push_str("        controller.setSystemBarsBehavior(\n");
    out.push_str(
        "                WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE);\n",
    );
    out.push_str("    }\n\n");
    out.push_str("    @Override\n");
    out.push_str("    protected void onCreate(Bundle savedInstanceState) {\n");
    out.push_str("        WindowCompat.setDecorFitsSystemWindows(getWindow(), false);\n");
    out.push_str("        hideSystemUI();\n");
    out.push_str("        super.onCreate(savedInstanceState);\n");
    out.push_str("    }\n\n");
    out.push_str("    protected void onResume() {\n");
    out.push_str("        super.onResume();\n");
    out.push_str("        hideSystemUI();\n");
    out.push_str("    }\n");
    out.push_str("}\n");
    out
}
