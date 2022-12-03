package com.tomicooler.tata.watchersms;

import android.app.IntentService;
import android.content.Intent;
import android.os.Bundle;
import android.telephony.PhoneNumberUtils;
import android.telephony.SmsMessage;

import com.tomicooler.tata.Serializer;
import com.tomicooler.tata.pojo.Protector;
import com.tomicooler.tata.watchercommon.CommandHandler;

public class SmsIntentService extends IntentService {

    public SmsIntentService() {
        super("SmsIntentService");
    }

    @Override
    protected void onHandleIntent(Intent intent) {
        final Bundle bundle = intent.getExtras();

        try {

            if (bundle != null) {
                final Object[] pdusObj = (Object[]) bundle.get("pdus");
                if (pdusObj != null) {
                    for (Object aPdusObj : pdusObj) {
                        SmsMessage currentMessage = SmsMessage.createFromPdu((byte[]) aPdusObj);
                        String phoneNumber = currentMessage.getDisplayOriginatingAddress();
                        String message = currentMessage.getDisplayMessageBody();

                        System.out.println("response: " + message);
                        Protector protector = getResponse(message, phoneNumber);
                        if (protector != null) {
                            CommandHandler commandHandler = new CommandHandler(getApplicationContext(), MainActivity.class, getApplicationContext().getString(R.string.app_name));
                            commandHandler.handleCommand(protector);
                        }
                    }
                }
            }

        } catch (Exception ignored) {
        }

        SmsBroadcastReceiver.completeWakefulIntent(intent);
    }

    private Protector getResponse(final String message, final String phoneNumber) {
        SMSPreferences preferences = new SMSPreferences(getApplicationContext());
        if (!PhoneNumberUtils.compare(preferences.getPairPhoneNumber(), phoneNumber)) {
            return null;
        }

        Protector protector = null;

        String[] strings = message.split("/");
        if (strings.length == 2) {
            if (strings[0].equals("$tATA")) {
                protector = Serializer.read(strings[1], Protector.class);
            }
        }
        return protector;
    }
}