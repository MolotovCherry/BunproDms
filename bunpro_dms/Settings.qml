import QtQuick
import qs.Modules.Plugins

PluginSettings {
    id: root
    pluginId: "bunpro"

    StringSetting {
        settingKey: "apiKey"
        label: "API Key"
        description: "Website or app API key (in Settings -> Bunpro API)."
        placeholder: "bunprojapanese1234567890abcdefgh"
        defaultValue: ""
    }

    ToggleSetting {
        settingKey: "dangerouslyAuthenticateUsingApiKey"
        label: "Use App API Key"
        description: "Use Bunpro 3rd party app API key instead of short lived website api key. [WARN: Key lives forever and should manually be rotated if leaked!]"
        defaultValue: true
    }

    SliderSetting {
        settingKey: "updateInterval"
        label: "Update Interval"
        description: "How often in mins to refresh data. Updates every X minutes starting from beginning of hour. E.g. 15 updates at 00/15/30/45 of every hour."
        defaultValue: 15
        minimum: 5
        maximum: 60
    }
}
