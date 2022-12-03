package com.tomicooler.tata.watchersms;

import android.Manifest;
import android.content.Context;
import android.content.pm.PackageManager;
import android.support.v4.content.ContextCompat;
import android.telephony.SmsManager;

import java.util.ArrayList;

class Telephoner {

    private final Context context;
    private final String number;

    Telephoner(Context context, String number) {
        this.context = context;
        this.number = number;
    }

    void sendSMS(final String message) {
        if (ContextCompat.checkSelfPermission(context,
                Manifest.permission.SEND_SMS)
                == PackageManager.PERMISSION_GRANTED) {

            SmsManager smsManager = SmsManager.getDefault();
            ArrayList<String> parts = smsManager.divideMessage(message);
            smsManager.sendMultipartTextMessage(number, null, parts, null, null);
        }
    }

}
