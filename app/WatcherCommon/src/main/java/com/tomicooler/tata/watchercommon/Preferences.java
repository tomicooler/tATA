package com.tomicooler.tata.watchercommon;

import android.content.Context;
import android.content.SharedPreferences;
import android.preference.PreferenceManager;

import com.tomicooler.tata.pojo.CarLocation;
import com.tomicooler.tata.Serializer;
import com.tomicooler.tata.pojo.ParkLocation;

public class Preferences {

    private static final String PROPERTY_CAR_LOCATION = "car_location";
    private static final String PROPERTY_PARK_LOCATION = "park_location";

    // http://androidblog.reindustries.com/check-if-an-android-activity-is-currently-running/
    private static final String PROPERTY_ACTIVITY_IS_RUNNING = "activity_is_running";

    public static final String PROPERTY_MY_PHONE_NUMBER = "my_phone_number";

    private static final String PROPERTY_SERVICE_ENABLED = "service_enabled";
    private static final String PROPERTY_LAST_UPDATE_TIME = "last_update_time";

    private static final String PROPERTY_LAST_UPDATE_WIHTOUT_LOCATION = "last_update_without_location";
    private static final String PROPERTY_LAST_UPDATE_REQUEST_TIME = "last_update_request_time";

    private final Context context;
    protected final SharedPreferences preferences;

    public Preferences(Context context) {
        this.context = context;
        this.preferences = getPreferences();
    }

    public CarLocation getCarLocation() {
        return Serializer.read(preferences.getString(PROPERTY_CAR_LOCATION, ""), CarLocation.class);
    }

    public ParkLocation getParkLocation() {
        return Serializer.read(preferences.getString(PROPERTY_PARK_LOCATION, ""), ParkLocation.class);
    }

    public long getLastUpdateTime() {
        return preferences.getLong(PROPERTY_LAST_UPDATE_TIME, 0);
    }

    public boolean isActivityRunning() {
        return preferences.getBoolean(PROPERTY_ACTIVITY_IS_RUNNING, false);
    }

    public boolean isServiceEnabled() {
        return preferences.getBoolean(PROPERTY_SERVICE_ENABLED, false);
    }

    public boolean wasLastUpdateWithoutLocation() {
        return preferences.getBoolean(PROPERTY_LAST_UPDATE_WIHTOUT_LOCATION, false);
    }

    public long getLastUpdateRequestTime() {
        return preferences.getLong(PROPERTY_LAST_UPDATE_REQUEST_TIME, 0);
    }

    public String getMyPhoneNumber() {
        return preferences.getString(PROPERTY_MY_PHONE_NUMBER, "");
    }

    public void storeCarLocation(CarLocation carLocation) {
        SharedPreferences.Editor editor = preferences.edit();
        editor.putString(PROPERTY_CAR_LOCATION, Serializer.write(carLocation));
        editor.apply();
    }

    public void storeParkLocation(ParkLocation parkLocation) {
        SharedPreferences.Editor editor = preferences.edit();
        editor.putString(PROPERTY_PARK_LOCATION, Serializer.write(parkLocation));
        editor.apply();
    }

    public void storeActivityIsRunning(boolean running) {
        SharedPreferences.Editor editor = preferences.edit();
        editor.putBoolean(PROPERTY_ACTIVITY_IS_RUNNING, running);
        editor.apply();
    }

    public void storeServiceEnabled(boolean enabled) {
        SharedPreferences.Editor editor = preferences.edit();
        editor.putBoolean(PROPERTY_SERVICE_ENABLED, enabled);
        editor.apply();
    }

    public void storeLastUpdateTime(long lastUpdateTime) {
        SharedPreferences.Editor editor = preferences.edit();
        editor.putLong(PROPERTY_LAST_UPDATE_TIME, lastUpdateTime);
        editor.apply();
    }

    public void storeLastUpdateWithoutLocation(boolean hasLocation) {
        SharedPreferences.Editor editor = preferences.edit();
        editor.putBoolean(PROPERTY_LAST_UPDATE_WIHTOUT_LOCATION, hasLocation);
        editor.apply();
    }

    public void storeLastUpdateRequestTime(long lastUpdateRequestTime) {
        SharedPreferences.Editor editor = preferences.edit();
        editor.putLong(PROPERTY_LAST_UPDATE_REQUEST_TIME, lastUpdateRequestTime);
        editor.apply();
    }

    private SharedPreferences getPreferences() {
        return PreferenceManager.getDefaultSharedPreferences(context);
    }
}
