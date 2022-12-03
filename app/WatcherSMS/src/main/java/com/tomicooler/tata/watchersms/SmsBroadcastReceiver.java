package com.tomicooler.tata.watchersms;

import android.app.Activity;
import android.content.ComponentName;
import android.content.Context;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.support.v4.content.WakefulBroadcastReceiver;

public class SmsBroadcastReceiver extends WakefulBroadcastReceiver {

    public void onReceive(Context context, Intent intent) {
        ComponentName comp = new ComponentName(context.getPackageName(),
                SmsIntentService.class.getName());
        startWakefulService(context, (intent.setComponent(comp)));
        setResultCode(Activity.RESULT_OK);
    }

    public static void enableSmsReceiver(Context context, boolean enable) {
        ComponentName receiver = new ComponentName(context, SmsBroadcastReceiver.class);
        PackageManager pm = context.getPackageManager();
        int enable_or_disable;

        if (enable) {
            enable_or_disable = PackageManager.COMPONENT_ENABLED_STATE_ENABLED;
        } else {
            enable_or_disable = PackageManager.COMPONENT_ENABLED_STATE_DISABLED;
        }

        pm.setComponentEnabledSetting(receiver,
                enable_or_disable,
                PackageManager.DONT_KILL_APP);
    }
}