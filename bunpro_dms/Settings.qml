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
        description: "Use Bunpro 3rd party app API key instead of ephemeral website api key. [WARN: Key lives forever and should manually be rotated if leaked!]"
        defaultValue: true
    }
}
