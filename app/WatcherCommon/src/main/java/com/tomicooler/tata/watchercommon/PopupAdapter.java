package com.tomicooler.tata.watchercommon;

import android.annotation.SuppressLint;
import android.view.LayoutInflater;
import android.view.View;
import android.widget.TextView;

import com.google.android.gms.maps.GoogleMap.InfoWindowAdapter;
import com.google.android.gms.maps.model.Marker;

class PopupAdapter implements InfoWindowAdapter {
    private View popup = null;
    private LayoutInflater inflater = null;

    PopupAdapter(LayoutInflater inflater) {
        this.inflater = inflater;
    }

    @Override
    public View getInfoWindow(Marker marker) {
        return (null);
    }

    @SuppressLint("InflateParams")
    @Override
    public View getInfoContents(Marker marker) {
        if (popup == null) {
            popup = inflater.inflate(R.layout.popup, null);
        }

        TextView tv = (TextView) popup.findViewById(R.id.popup_title);

        tv.setText(marker.getTitle());
        tv = (TextView) popup.findViewById(R.id.popup_snippet);
        tv.setText(marker.getSnippet());

        return popup;
    }
}