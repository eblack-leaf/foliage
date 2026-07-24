package io.github.eblackleaf.application;

import androidx.core.view.WindowCompat;
import androidx.core.view.WindowInsetsCompat;
import androidx.core.view.WindowInsetsControllerCompat;

import com.google.androidgamesdk.GameActivity;

import android.os.Bundle;
import android.view.View;
import android.view.WindowManager;

public class MainActivity extends GameActivity {

    static {
        // Must match both the foliage app's `[lib] name` (its cdylib output) and
        // the `android.app.lib_name` meta-data value in AndroidManifest.xml -- all
        // three have to agree, or the JVM can't find `android_main` to call into.
        System.loadLibrary("application");
    }

    private void hideSystemUI() {
        getWindow().getAttributes().layoutInDisplayCutoutMode
                = WindowManager.LayoutParams.LAYOUT_IN_DISPLAY_CUTOUT_MODE_ALWAYS;
        View decorView = getWindow().getDecorView();
        WindowInsetsControllerCompat controller = new WindowInsetsControllerCompat(getWindow(),
                decorView);
        controller.hide(WindowInsetsCompat.Type.systemBars());
        controller.hide(WindowInsetsCompat.Type.displayCutout());
        controller.setSystemBarsBehavior(
                WindowInsetsControllerCompat.BEHAVIOR_SHOW_TRANSIENT_BARS_BY_SWIPE);
    }

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        WindowCompat.setDecorFitsSystemWindows(getWindow(), false);
        hideSystemUI();
        super.onCreate(savedInstanceState);
    }

    protected void onResume() {
        super.onResume();
        hideSystemUI();
    }
}
