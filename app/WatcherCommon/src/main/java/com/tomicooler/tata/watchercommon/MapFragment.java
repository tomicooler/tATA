package com.tomicooler.tata.watchercommon;

import android.Manifest;
import android.app.Activity;
import android.app.Fragment;
import android.content.BroadcastReceiver;
import android.content.Context;
import android.content.Intent;
import android.content.IntentFilter;
import android.content.pm.PackageManager;
import android.graphics.Color;
import android.net.Uri;
import android.os.Bundle;
import android.support.annotation.NonNull;
import android.support.v4.content.ContextCompat;
import android.support.v4.content.LocalBroadcastManager;
import android.text.format.DateUtils;
import android.util.DisplayMetrics;
import android.view.LayoutInflater;
import android.view.View;
import android.view.ViewGroup;
import android.widget.ImageButton;
import android.widget.Toast;
import android.widget.ToggleButton;

import com.google.android.gms.maps.OnMapReadyCallback;
import com.tomicooler.tata.pojo.Call;
import com.tomicooler.tata.pojo.CarLocation;
import com.tomicooler.tata.pojo.Park;
import com.tomicooler.tata.pojo.ParkLocation;
import com.tomicooler.tata.pojo.Refresh;
import com.tomicooler.tata.pojo.Service;
import com.tomicooler.tata.pojo.Watcher;
import com.google.android.gms.maps.CameraUpdateFactory;
import com.google.android.gms.maps.GoogleMap;
import com.google.android.gms.maps.MapView;
import com.google.android.gms.maps.MapsInitializer;
import com.google.android.gms.maps.model.CircleOptions;
import com.google.android.gms.maps.model.LatLng;
import com.google.android.gms.maps.model.Marker;
import com.google.android.gms.maps.model.MarkerOptions;

public class MapFragment extends Fragment implements OnMapReadyCallback {

    public interface Communicator {
        void sendMessage(Watcher watcher);

        boolean isConfigured();
    }

    private static final String SAVE_STATE_BUTTONS_ENABLED = "save_state_buttons_enabled";
    private static final String SAVE_STATE_PROGRESS_ENABLED = "save_state_progress_enabled";
    private static final String ARGUMENT_NO_FIRST_UPDATE = "argument_no_first_update";

    private MapView mMapView = null;
    private GoogleMap mMap = null;
    private Bundle mBundle = null;
    private Context context = null;
    private Preferences preferences = null;
    private boolean buttonsEnabled;
    private boolean progressEnabled;
    private boolean noFirstUpdate;

    private UpdateReceiver updateReceiver;
    private IntentFilter intentFilter;
    private Marker carMarker;

    private Communicator communicator;

    public static MapFragment newInstance(boolean noFirstUpdate) {
        MapFragment mapFragment = new MapFragment();
        Bundle arguments = new Bundle();
        arguments.putBoolean(ARGUMENT_NO_FIRST_UPDATE, noFirstUpdate);
        mapFragment.setArguments(arguments);
        return mapFragment;
    }

    @Override
    public void onCreate(Bundle savedInstanceState) {
        super.onCreate(savedInstanceState);

        noFirstUpdate = getArguments().getBoolean(ARGUMENT_NO_FIRST_UPDATE);
    }

    @Override
    public View onCreateView(LayoutInflater inflater, ViewGroup container,
                             Bundle savedInstanceState) {
        View inflatedView = inflater.inflate(R.layout.fragment_map, container, false);

        MapsInitializer.initialize(getActivity());

        mMapView = (MapView) inflatedView.findViewById(R.id.map);
        mMapView.onCreate(mBundle);
        setUpMapIfNeeded(inflatedView);

        final ToggleButton parker = (ToggleButton) inflatedView.findViewById(R.id.switchParkingSV);
        final ToggleButton service = (ToggleButton) inflatedView.findViewById(R.id.serviceToggle);

        View.OnClickListener onClickListener = new View.OnClickListener() {
            @Override
            public void onClick(View v) {
                int i = v.getId();
                if (i == R.id.buttonNavigate) {
                    startNavigateToCar();

                } else if (i == R.id.buttonCar) {
                    showCarOnMap();

                } else if (i == R.id.buttonPark) {
                    showParkOnMap();

                } else if (i == R.id.buttonRefresh) {
                    sendRefresh();

                } else if (i == R.id.buttonMaybe) {
                    sendCallMe();

                } else if (i == R.id.switchParkingSV) {
                    parker.setChecked(!parker.isChecked());
                    if (parker.isChecked()) {
                        sendParkOff();
                    } else {
                        sendParkOn();
                    }

                } else if (i == R.id.serviceToggle) {
                    service.setChecked(!service.isChecked());
                    if (service.isChecked()) {
                        sendServiceOff();
                    } else {
                        sendServiceOn();
                    }

                }
            }
        };

        inflatedView.findViewById(R.id.buttonNavigate).setOnClickListener(onClickListener);
        inflatedView.findViewById(R.id.buttonCar).setOnClickListener(onClickListener);
        inflatedView.findViewById(R.id.buttonPark).setOnClickListener(onClickListener);
        inflatedView.findViewById(R.id.buttonRefresh).setOnClickListener(onClickListener);
        inflatedView.findViewById(R.id.buttonMaybe).setOnClickListener(onClickListener);
        parker.setOnClickListener(onClickListener);
        service.setOnClickListener(onClickListener);

        buttonsEnabled = true;

        return inflatedView;
    }

    private void sendServiceOn() {
        Toast.makeText(context, context.getString(R.string.map_requesting_service_on), Toast.LENGTH_LONG).show();
        Watcher watcher = new Watcher();
        watcher.service = new Service();
        watcher.service.service = true;
        sendMessage(watcher);
    }

    private void sendServiceOff() {
        Toast.makeText(context, context.getString(R.string.map_requesting_service_off), Toast.LENGTH_LONG).show();
        Watcher watcher = new Watcher();
        watcher.service = new Service();
        watcher.service.service = false;
        sendMessage(watcher);
    }

    private void showParkOnMap() {
        ParkLocation parkLocation = preferences.getParkLocation();
        if (parkLocation != null && mMap != null) {
            mMap.moveCamera(CameraUpdateFactory.newLatLngZoom(new LatLng(parkLocation.position.latitude, parkLocation.position.longitude), (float) getZoomLevel(parkLocation.accuracy)));
        }
    }

    private void showCarOnMap() {
        CarLocation carLocation = preferences.getCarLocation();
        if (carLocation != null && mMap != null) {
            mMap.moveCamera(CameraUpdateFactory.newLatLngZoom(new LatLng(carLocation.position.latitude, carLocation.position.longitude), (float) getZoomLevel(carLocation.accuracy)));
            if (carMarker != null) {
                if (!carMarker.isInfoWindowShown()) {
                    carMarker.showInfoWindow();
                }
            }
        }
    }

    @Override
    public void onActivityCreated(Bundle savedInstanceState) {
        super.onActivityCreated(savedInstanceState);

        mBundle = savedInstanceState;
        context = getActivity().getApplicationContext();
        preferences = new Preferences(context);
        updateReceiver = new UpdateReceiver();
        intentFilter = new IntentFilter();
        intentFilter.addAction(CommandHandler.ACTION_UPDATE);
        progressEnabled = false;

        if (savedInstanceState != null) {
            boolean buttonsWereEnabledBefore = savedInstanceState.getBoolean(SAVE_STATE_BUTTONS_ENABLED, true);
            boolean progressWasEnabledBefore = savedInstanceState.getBoolean(SAVE_STATE_PROGRESS_ENABLED, false);
            enableProgressBar(progressWasEnabledBefore);
            enableButtons(buttonsWereEnabledBefore);
        } else {
            boolean isConfigured = communicator != null && communicator.isConfigured();
            if (!noFirstUpdate && isConfigured) {
                long currentTime = System.currentTimeMillis();
                long lastTimestamp = preferences.getLastUpdateTime();
                long lastRequestTimestamp = preferences.getLastUpdateRequestTime();
                lastTimestamp = Math.max(lastTimestamp, lastRequestTimestamp);
                if (Math.abs(currentTime - lastTimestamp) > 10 * 60 * 1000) {
                    // This is not running once per App lifecycle :(
                    sendRefresh();
                }
            }
            if (!isConfigured) {
                enableProgressBar(false);
            }
        }
    }

    @Override
    public void onAttach(Activity activity) {
        super.onAttach(activity);
        try {
            communicator = (Communicator) activity;
        } catch (ClassCastException e) {
            communicator = null;
        }
    }

    @Override
    public void onSaveInstanceState(@NonNull Bundle outState) {
        super.onSaveInstanceState(outState);
        outState.putBoolean(SAVE_STATE_BUTTONS_ENABLED, buttonsEnabled);
        outState.putBoolean(SAVE_STATE_PROGRESS_ENABLED, progressEnabled);
    }

    @Override
    public void onResume() {
        super.onResume();
        mMapView.onResume();
        setUpMapIfNeeded(getView());
        update();
        if (preferences.getLastUpdateRequestTime() != 0 &&
                (preferences.getLastUpdateRequestTime() < preferences.getLastUpdateTime())) {
            CommandHandler.cancelNotification(context);
            enableProgressBar(false);
            enableButtons(true);
        }
    }

    @Override
    public void onStart() {
        super.onStart();
        LocalBroadcastManager.getInstance(context).registerReceiver(updateReceiver, intentFilter);
    }

    @Override
    public void onStop() {
        super.onStop();
        LocalBroadcastManager.getInstance(context).unregisterReceiver(updateReceiver);
    }

    public void onError(String error) {
        Toast.makeText(context, error, Toast.LENGTH_LONG).show();
        enableProgressBar(false);
        enableButtons(true);
        preferences.storeLastUpdateRequestTime(0);
    }

    private void update() {
        CarLocation carLocation = preferences.getCarLocation();
        ParkLocation parkLocation = preferences.getParkLocation();
        boolean isParking = parkLocation != null;
        boolean isServiceEnabled = preferences.isServiceEnabled();
        if (carLocation != null) {
            setMarkerOnPosition(carLocation, parkLocation, preferences.wasLastUpdateWithoutLocation());
            updateUI(getView(), isParking, isServiceEnabled);
            showCarOnMap();
        }
    }

    @Override
    public void onPause() {
        super.onPause();
        mMapView.onPause();
        mMap = null;
    }

    @Override
    public void onDestroy() {
        mMapView.onDestroy();
        super.onDestroy();
    }

    private void sendParkOn() {
        Toast.makeText(context, context.getString(R.string.map_requesting_park_on), Toast.LENGTH_LONG).show();
        Watcher watcher = new Watcher();
        watcher.park = new Park();
        watcher.park.park = true;
        sendMessage(watcher);
    }

    private void sendParkOff() {
        Toast.makeText(context, context.getString(R.string.map_requesting_park_off), Toast.LENGTH_LONG).show();
        Watcher watcher = new Watcher();
        watcher.park = new Park();
        watcher.park.park = false;
        sendMessage(watcher);
    }

    private void sendCallMe() {
        Toast.makeText(context, context.getString(R.string.map_requesting_call_me), Toast.LENGTH_LONG).show();
        Watcher watcher = new Watcher();
        watcher.call = new Call();
        watcher.call.call = true;
        sendMessage(watcher);
    }

    private void sendRefresh() {
        Toast.makeText(context, context.getString(R.string.map_requesting_update), Toast.LENGTH_LONG).show();
        Watcher watcher = new Watcher();
        watcher.refresh = new Refresh();
        watcher.refresh.refresh = true;
        sendMessage(watcher);
    }

    private void setUpMapIfNeeded(View inflatedView) {
        if (mMap == null) {
            ((MapView) inflatedView.findViewById(R.id.map)).getMapAsync(this);
        }
    }

    @Override
    public void onMapReady(GoogleMap googleMap) {
        mMap = googleMap;
        mMap.setInfoWindowAdapter(new PopupAdapter(getActivity().getLayoutInflater()));
        setMyLocationEnabled();
        update();
    }

    public void setMyLocationEnabled() {
        if (ContextCompat.checkSelfPermission(context,
                Manifest.permission.ACCESS_FINE_LOCATION)
                == PackageManager.PERMISSION_GRANTED) {
            if (mMap != null) {
                mMap.setMyLocationEnabled(true);
            }
        }
    }

    public void disableButtons() {
        Toast.makeText(context, context.getString(R.string.map_disable_buttons), Toast.LENGTH_LONG).show();
        enableButtons(false);
    }

    private void setMarkerOnPosition(final CarLocation carLocation, final ParkLocation parkLocation, boolean wasLastUpdateWithoutCarLocation) {
        if (mMap == null) {
            return;
        }

        mMap.clear();

        if (carLocation != null) {
            carMarker = mMap.addMarker(new MarkerOptions()
                    .title(context.getString(R.string.map_title_car_position))
                    .snippet(String.format("%s\n%s: %.2f %%%s",
                            millisecondsToString(carLocation.timestamp),
                            context.getString(R.string.map_battery),
                            carLocation.battery * 100.0f,
                            wasLastUpdateWithoutCarLocation ? "\n" + context.getString(R.string.last_update_without_location) : ""))
                    .position(new LatLng(carLocation.position.latitude, carLocation.position.longitude)));

            addCircle(new LatLng(carLocation.position.latitude, carLocation.position.longitude), carLocation.accuracy, false);
        }
        if (parkLocation != null) {
            addCircle(new LatLng(parkLocation.position.latitude, parkLocation.position.longitude), parkLocation.accuracy, true);
        }
    }

    private void addCircle(LatLng position, float accuracy, boolean isParking) {
        CircleOptions co = new CircleOptions();
        co.center(position);
        co.radius(accuracy);

        if (isParking) {
            co.fillColor(Color.argb(100, 0, 255, 0));
            co.strokeColor(Color.argb(255, 0, 255, 0));
        } else {
            co.fillColor(Color.argb(100, 0, 0, 255));
            co.strokeColor(Color.argb(255, 0, 0, 255));
        }

        co.strokeWidth(4.0f);

        mMap.addCircle(co);
    }

    private double getZoomLevel(double radius) {
        DisplayMetrics metrics = getResources().getDisplayMetrics();
        double width = (double) Math.min(mMapView.getWidth(), mMapView.getHeight()) / (double) metrics.scaledDensity;

        double scale = Math.max(radius, 200.0) / Math.max(300.0, width);
        return (16.0 - Math.log(scale) / Math.log(2.0));
    }

    private void updateUI(View view, boolean isParked, boolean isServiceEnabled) {
        if (view != null) {
            ToggleButton parker = (ToggleButton) view.findViewById(R.id.switchParkingSV);

            if (parker != null) {
                parker.setChecked(isParked);
            }

            ImageButton showPark = (ImageButton) view.findViewById(R.id.buttonPark);

            if (showPark != null) {
                showPark.setEnabled(isParked);
            }

            ToggleButton service = (ToggleButton) view.findViewById(R.id.serviceToggle);

            if (service != null) {
                service.setChecked(isServiceEnabled);
            }
        }
    }

    private String millisecondsToString(long millis) {
        if (millis == 0) {
            return context.getString(R.string.map_unknown);
        } else {
            return DateUtils.formatDateTime(context, millis, DateUtils.FORMAT_SHOW_TIME | DateUtils.FORMAT_SHOW_DATE);
        }
    }

    private void startNavigateToCar() {
        CarLocation location = preferences.getCarLocation();
        if (location != null) {
            Intent navigation = new Intent(Intent.ACTION_VIEW,
                    Uri.parse("google.navigation:q=" +
                                    String.valueOf(location.position.latitude) +
                                    "," +
                                    String.valueOf(location.position.longitude)
                    )
            );
            startActivity(navigation);
        }
    }

    private void enableButtons(boolean enable) {
        View view = getView();
        if (view != null) {
            view.findViewById(R.id.buttonRefresh).setEnabled(enable);
            view.findViewById(R.id.buttonMaybe).setEnabled(enable);
            view.findViewById(R.id.switchParkingSV).setEnabled(enable);
            view.findViewById(R.id.serviceToggle).setEnabled(enable);
            buttonsEnabled = enable;
        }
    }

    private void enableProgressBar(boolean enable) {
        final Activity activity = getActivity();
        if (activity != null) {
            activity.setProgressBarIndeterminateVisibility(enable);
            progressEnabled = enable;
        }
    }

    private void sendMessage(Watcher message) {
        preferences.storeLastUpdateRequestTime(System.currentTimeMillis());
        enableProgressBar(true);
        enableButtons(false);
        if (communicator != null) {
            communicator.sendMessage(message);
        }
    }

    public class UpdateReceiver extends BroadcastReceiver {
        @Override
        public void onReceive(Context context, Intent intent) {
            if (intent.getAction() == null) {
                return;
            }

            update();
            enableProgressBar(false);
            enableButtons(true);
            preferences.storeLastUpdateRequestTime(0);
        }
    }
}