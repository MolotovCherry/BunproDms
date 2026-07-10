pragma Singleton
pragma ComponentBehavior: Bound

import QtQuick

import qs.Services

import "../bunpro"

Item {
    id: root

    Bunpro {
        id: bunpro
        apiKey: ""
        onError: error => ToastService.showError("Bunpro Error", error)
    }

    property alias apiKey: bunpro.apiKey
    property alias dangerouslyAuthenticateUsingApiKey: bunpro.dangerouslyAuthenticateUsingApiKey
    readonly property alias current: bunpro.current
    property alias updateInterval: bunpro.updateInterval

    function refreshForecast() {
        if (apiKey !== "") {
            bunpro.refreshForecast();
        }
    }
}
