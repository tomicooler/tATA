package com.tomicooler.tata.watchersms;

import android.content.Context;
import android.content.SharedPreferences;

import com.tomicooler.tata.watchercommon.Preferences;

class SMSPreferences extends Preferences {
    public static final String PROPERTY_PAIR_PHONE_NUMBER = "pair_phone_number";
    public static final String PROPERTY_SMS_PASSWORD = "sms_password";
    public static final String PROPERTY_HAS_SMS_PERMISSION = "sms_permission";

    public SMSPreferences(Context context) {
        super(context);
    }

    public String getPairPhoneNumber() {
        return preferences.getString(PROPERTY_PAIR_PHONE_NUMBER, "");
    }

    public String getSMSPassword() {
        return preferences.getString(PROPERTY_SMS_PASSWORD, "");
    }

    public boolean hasSMSPermission() {
        return preferences.getBoolean(PROPERTY_HAS_SMS_PERMISSION, true);
    }

    public void storeSMSPermission(boolean has) {
        SharedPreferences.Editor editor = preferences.edit();
        editor.putBoolean(PROPERTY_HAS_SMS_PERMISSION, has);
        editor.apply();
    }
}
