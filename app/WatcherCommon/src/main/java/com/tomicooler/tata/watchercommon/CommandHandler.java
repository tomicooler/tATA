package com.tomicooler.tata.watchercommon;

import android.app.NotificationChannel;
import android.app.NotificationManager;
import android.app.PendingIntent;
import android.content.Context;
import android.content.Intent;
import android.os.Build;
import android.support.v4.app.NotificationCompat;
import android.support.v4.content.LocalBroadcastManager;

import com.tomicooler.tata.pojo.Protector;

public class CommandHandler {

    private static final int NOTIFICATION_ID = 1;
    public static final String ACTION_UPDATE = "com.tomicooler.tata.watchercommon.UPDATE";
    public static final String STARTED_FROM_NOTIFICATION = "started_from_notification";

    private final Context context;
    private final Class<?> clazz;
    private final String appName;

    public CommandHandler(final Context context, final Class<?> clazz, final String appName) {
        this.context = context;
        this.clazz = clazz;
        this.appName = appName;
    }

    public void handleCommand(final Protector protector) {
        if (protector == null) {
            return;
        }

        Preferences preferences = new Preferences(context);

        preferences.storeLastUpdateTime(System.currentTimeMillis());

        if (protector.carLocation != null) {
            preferences.storeLastUpdateWithoutLocation(false);
            preferences.storeCarLocation(protector.carLocation);
        } else {
            preferences.storeLastUpdateWithoutLocation(true);
        }

        preferences.storeParkLocation(protector.parkLocation);

        if (protector.service != null) {
            preferences.storeServiceEnabled(protector.service.service);
        }

        if (preferences.isActivityRunning()) {
            Intent saIntent = new Intent(ACTION_UPDATE);
            LocalBroadcastManager.getInstance(context).sendBroadcast(saIntent);
        } else {
            String notification = context.getString(R.string.notification_text);
            if (protector.status != null) {
                switch (protector.status.status) {
                    case PARKING_DETECTED:
                        notification = context.getString(R.string.status_park_detected);
                        break;
                    case PARKING_UPDATED:
                        notification = context.getString(R.string.status_park_location_updated);
                        break;
                    case CAR_THEFT_DETECTED:
                        notification = context.getString(R.string.status_car_theft);
                        break;
                }
            }
            sendNotification(notification);
        }
    }

    private void sendNotification(String msg) {
        NotificationManager mNotificationManager = (NotificationManager)
                context.getSystemService(Context.NOTIFICATION_SERVICE);

        Intent intent = new Intent(context, clazz);
        intent.putExtra(STARTED_FROM_NOTIFICATION, true);
        PendingIntent contentIntent = PendingIntent.getActivity(context, 0,
                intent, PendingIntent.FLAG_UPDATE_CURRENT);


        NotificationCompat.Builder mBuilder;
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            String CHANNEL_ID = "TATA_WATCHER_COMMON_CHANNEL_0";
            NotificationChannel channel = new NotificationChannel(CHANNEL_ID,
                    appName,
                    NotificationManager.IMPORTANCE_DEFAULT);

            ((NotificationManager) context.getSystemService(Context.NOTIFICATION_SERVICE)).createNotificationChannel(channel);

            mBuilder = new NotificationCompat.Builder(context, CHANNEL_ID);
        } else {
            mBuilder = new NotificationCompat.Builder(context);
        }

        mBuilder.setSmallIcon(R.drawable.ic_notification)
                        .setContentTitle(appName)
                        .setStyle(new NotificationCompat.BigTextStyle()
                                .bigText(msg))
                        .setContentText(msg);

        mBuilder.setDefaults(NotificationCompat.DEFAULT_VIBRATE | NotificationCompat.DEFAULT_LIGHTS | NotificationCompat.DEFAULT_SOUND);
        mBuilder.setContentIntent(contentIntent);
        mBuilder.setAutoCancel(true);
        mNotificationManager.notify(NOTIFICATION_ID, mBuilder.build());
    }

    public static void cancelNotification(Context context) {
        NotificationManager mNotificationManager = (NotificationManager)
                context.getSystemService(Context.NOTIFICATION_SERVICE);
        mNotificationManager.cancel(NOTIFICATION_ID);
    }
}
