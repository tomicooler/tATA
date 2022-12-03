package com.tomicooler.tata.watchersms;

import android.app.Activity;
import android.os.Bundle;

import com.tomicooler.tata.watchercommon.R;

public class SettingsActivity extends Activity {
    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);
        setContentView(R.layout.activity_settings);
        if (findViewById(R.id.settings_content) != null) {
            if (savedInstanceState != null) {
                return;
            }

            PreferencesFragment preferencesFragment = new PreferencesFragment();
            getFragmentManager().beginTransaction().add(R.id.settings_content, preferencesFragment).commit();
        }
    }
}
