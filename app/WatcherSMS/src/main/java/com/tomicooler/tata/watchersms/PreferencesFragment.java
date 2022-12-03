package com.tomicooler.tata.watchersms;

import android.content.Context;
import android.content.Intent;
import android.content.SharedPreferences;
import android.net.Uri;
import android.os.Bundle;
import android.preference.EditTextPreference;
import android.preference.Preference;
import android.preference.PreferenceFragment;
import android.util.Patterns;

import com.tomicooler.tata.core.AboutActivity;

public class PreferencesFragment extends PreferenceFragment implements SharedPreferences.OnSharedPreferenceChangeListener {
    private SMSPreferences preferences;
    private Context context;

    @Override
    public void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        addPreferencesFromResource(R.xml.preferences);

        findPreference(SMSPreferences.PROPERTY_PAIR_PHONE_NUMBER).setOnPreferenceChangeListener(new Preference.OnPreferenceChangeListener() {
            @Override
            public boolean onPreferenceChange(Preference preference, Object newValue) {
                return isValidPhoneNumber((String) newValue);
            }
        });

        findPreference(SMSPreferences.PROPERTY_SMS_PASSWORD).setOnPreferenceChangeListener(new Preference.OnPreferenceChangeListener() {
            @Override
            public boolean onPreferenceChange(Preference preference, Object newValue) {
                return isValidPassword((String) newValue);
            }
        });

        findPreference("about").setOnPreferenceClickListener(new Preference.OnPreferenceClickListener() {
            @Override
            public boolean onPreferenceClick(Preference arg0) {
                showAbout();
                return true;
            }
        });

        findPreference("privacy").setOnPreferenceClickListener(new Preference.OnPreferenceClickListener() {
            @Override
            public boolean onPreferenceClick(Preference arg0) {
                showPrivacy();
                return true;
            }
        });
    }

    @Override
    public void onActivityCreated(Bundle savedInstanceState) {
        super.onActivityCreated(savedInstanceState);
        context = getActivity().getApplicationContext();
        preferences = new SMSPreferences(context);
        updatePhoneNumberSummary();
        updateSmsPasswordSummary();
        updatePreferencesWithSMSPermission();
    }

    @Override
    public void onResume() {
        super.onResume();
        getPreferenceManager().getSharedPreferences().registerOnSharedPreferenceChangeListener(this);
    }

    @Override
    public void onPause() {
        super.onPause();
        getPreferenceManager().getSharedPreferences().unregisterOnSharedPreferenceChangeListener(this);
    }

    @Override
    public void onSharedPreferenceChanged(SharedPreferences sharedPreferences, String key) {
        switch (key) {
            case SMSPreferences.PROPERTY_PAIR_PHONE_NUMBER:
                updatePhoneNumberSummary();
                break;
            case SMSPreferences.PROPERTY_SMS_PASSWORD:
                updateSmsPasswordSummary();
                break;
        }
    }

    private void updatePreferencesWithSMSPermission() {
        boolean enable = preferences.hasSMSPermission();
        findPreference(SMSPreferences.PROPERTY_SMS_PASSWORD).setEnabled(enable);
        findPreference(SMSPreferences.PROPERTY_PAIR_PHONE_NUMBER).setEnabled(enable);
    }

    private void updatePhoneNumberSummary() {
        String summary = preferences.getPairPhoneNumber();
        if (summary.isEmpty()) {
            summary = context.getString(R.string.preferences_summary_pair_phone_number);
        }
        EditTextPreference pref = (EditTextPreference) findPreference(SMSPreferences.PROPERTY_PAIR_PHONE_NUMBER);
        pref.setSummary(summary);
    }

    private void updateSmsPasswordSummary() {
        String summary = preferences.getSMSPassword();
        if (summary.isEmpty()) {
            summary = context.getString(R.string.preferences_summary_sms_password);
        }
        EditTextPreference pref = (EditTextPreference) findPreference(SMSPreferences.PROPERTY_SMS_PASSWORD);
        pref.setSummary(summary);
    }

    private void showAbout() {
        Intent intent = new Intent(context, AboutActivity.class);
        startActivity(intent);
    }

    private void showPrivacy() {
        Intent i = new Intent(Intent.ACTION_VIEW);
        i.setData(Uri.parse("https://github.com/tomicooler/tATAPP/tree/master/Watcher%20SMS/README.md"));
        startActivity(i);
    }

    private boolean isValidPhoneNumber(String phoneNumber) {
        return phoneNumber.isEmpty() || Patterns.PHONE.matcher(phoneNumber).matches();
    }

    private boolean isValidPassword(String password) {
        return !password.contains("/") && (password.length() >= 4) && (password.length() <= 20);
    }
}
