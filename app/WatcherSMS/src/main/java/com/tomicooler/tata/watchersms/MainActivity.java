package com.tomicooler.tata.watchersms;

import android.Manifest;
import android.app.Activity;
import android.content.Intent;
import android.content.pm.PackageManager;
import android.os.Bundle;
import android.support.annotation.NonNull;
import android.support.v4.app.ActivityCompat;
import android.view.Menu;
import android.view.MenuInflater;
import android.view.MenuItem;
import android.view.Window;

import com.tomicooler.tata.Serializer;
import com.tomicooler.tata.pojo.Watcher;
import com.tomicooler.tata.watchercommon.CommandHandler;
import com.tomicooler.tata.watchercommon.MapFragment;

import com.tomicooler.tata.watchercommon.R;

public class MainActivity extends Activity implements MapFragment.Communicator {

    private final static int MY_PERMISSION_REQUEST = 14;

    private SMSPreferences preferences;
    private Telephoner telephoner;

    private MapFragment mapFragment;

    @Override
    protected void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        requestWindowFeature(Window.FEATURE_INDETERMINATE_PROGRESS);

        preferences = new SMSPreferences(getApplicationContext());

        final Thread.UncaughtExceptionHandler defaultHandler = Thread.getDefaultUncaughtExceptionHandler();

        Thread.setDefaultUncaughtExceptionHandler(new Thread.UncaughtExceptionHandler() {
            @Override
            public void uncaughtException(Thread thread, Throwable throwable) {
                preferences.storeActivityIsRunning(false);
                defaultHandler.uncaughtException(thread, throwable);
            }
        });


        setContentView(R.layout.activity_main);

        telephoner = new Telephoner(getApplicationContext(), preferences.getPairPhoneNumber());

        boolean startedFromNotification = false;
        Intent intent = getIntent();
        if (intent != null) {
            Bundle extras = intent.getExtras();
            if (extras != null) {
                startedFromNotification = extras.getBoolean(CommandHandler.STARTED_FROM_NOTIFICATION, false);
            }
        }

        mapFragment = null;
        if (findViewById(R.id.main_content) != null) {
            if (savedInstanceState != null) {
                return;
            }

            mapFragment = MapFragment.newInstance(startedFromNotification);
            getFragmentManager().beginTransaction().add(R.id.main_content, mapFragment).commit();
        }

        if (!startedFromNotification) {
            requestDangerousPermissions();
        }

        if (android.os.Build.VERSION.SDK_INT < android.os.Build.VERSION_CODES.M) {
            if (!isConfigured()) {
                openSettings();
            }
        }
    }

    private void requestDangerousPermissions() {
        ActivityCompat.requestPermissions(this,
                new String[]{Manifest.permission.ACCESS_FINE_LOCATION, Manifest.permission.SEND_SMS, Manifest.permission.RECEIVE_SMS},
                MY_PERMISSION_REQUEST);
    }

    @Override
    protected void onResume() {
        super.onResume();
        telephoner = new Telephoner(getApplicationContext(), preferences.getPairPhoneNumber());
    }

    @Override
    protected void onStart() {
        super.onStart();
        preferences.storeActivityIsRunning(true);
    }

    @Override
    protected void onStop() {
        super.onStop();
        preferences.storeActivityIsRunning(false);
    }

    @Override
    public boolean onCreateOptionsMenu(Menu menu) {
        MenuInflater inflater = getMenuInflater();
        inflater.inflate(R.menu.main_activity_actions, menu);
        return super.onCreateOptionsMenu(menu);
    }

    @Override
    public boolean onOptionsItemSelected(MenuItem item) {
        int i = item.getItemId();
        if (i == R.id.action_settings) {
            openSettings();
            return true;
        } else {
            return super.onOptionsItemSelected(item);
        }
    }

    private void openSettings() {
        Intent intent = new Intent(this, SettingsActivity.class);
        startActivity(intent);
    }

    @Override
    public void sendMessage(Watcher watcher) {
        String msg = "$TATA/" + Serializer.write(watcher) + "/" + preferences.getSMSPassword();
        System.out.println("sending: " + msg);
        telephoner.sendSMS(msg);
    }

    @Override
    public boolean isConfigured() {
        return preferences.hasSMSPermission() && hasConfig();
    }

    private boolean hasConfig() {
        return !preferences.getSMSPassword().isEmpty() && !preferences.getPairPhoneNumber().isEmpty();
    }

    @Override
    public void onRequestPermissionsResult(int requestCode, @NonNull String permissions[],
                                           @NonNull int[] grantResults) {
        switch (requestCode) {
            case MY_PERMISSION_REQUEST: {
                for (int i = 0; i < permissions.length; ++i) {
                    if (permissions[i].equals(Manifest.permission.ACCESS_FINE_LOCATION) &&
                            grantResults[i] == PackageManager.PERMISSION_GRANTED) {
                        if (mapFragment != null) {
                            mapFragment.setMyLocationEnabled();
                        }
                    }
                    if (permissions[i].equals(Manifest.permission.SEND_SMS) &&
                            grantResults[i] == PackageManager.PERMISSION_DENIED) {
                        if (mapFragment != null) {
                            mapFragment.disableButtons();
                        }
                        preferences.storeSMSPermission(false);
                    } else {
                        preferences.storeSMSPermission(true);
                    }
                }

                if (preferences.hasSMSPermission() && !hasConfig()) {
                    openSettings();
                }
            }
        }
    }
}
