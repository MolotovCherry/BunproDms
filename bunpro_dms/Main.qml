pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts

import qs.Common
import qs.Widgets
import qs.Modules.Plugins
import qs.Services

import "./bunpro"

PluginComponent {
    id: root

    layerNamespacePlugin: "bunpro"

    property Bunpro bunpro: Bunpro {
        apiKey: pluginData?.apiKey || ""
        dangerouslyAuthenticateUsingApiKey: pluginData?.dangerouslyAuthenticateUsingApiKey !== undefined ? pluginData.dangerouslyAuthenticateUsingApiKey : true

        onError: error => ToastService.showError("Bunpro Error", error)
    }

    Connections {
        target: root.pluginService
        function onPluginDataChanged(changedId) {
            if (changedId !== root.layerNamespacePlugin) {
                return;
            }

            const oldKey = root.bunpro.apiKey;
            const newKey = root.pluginService.loadPluginData(root.layerNamespacePlugin, "apiKey", "");
            const oldDangerousLoad = root.bunpro.dangerouslyAuthenticateUsingApiKey;
            const newDangerousLoad = root.pluginService.loadPluginData(root.layerNamespacePlugin, "dangerouslyAuthenticateUsingApiKey", true);
            // settings changed, so we should try to reload data
            const shouldReload = (oldKey != newKey) || (oldDangerousLoad !== newDangerousLoad);

            if (shouldReload && (newKey !== "")) {
                root.bunpro.apiKey = newKey;
                root.bunpro.dangerouslyAuthenticateUsingApiKey = newDangerousLoad;
                console.info("bunpro setting changed; refreshing forecast");
                root.bunpro.refreshForecast();
            }
        }

        function onPluginLoaded(pluginId) {
            if (pluginId !== root.layerNamespacePlugin) {
                return;
            }

            if (root.bunpro.apiKey !== "") {
                console.info("bunpro settings loaded; refreshing forecast");
                root.bunpro.refreshForecast();
            }
        }
    }

    pillRightClickAction: (x, y, width, section, screen) => {
        if (root.bunpro.apiKey !== "") {
            console.info("bunpro right clicked; refreshing forecast");
            root.bunpro.refreshForecast();
        }
    }

    horizontalBarPill: Component {
        Row {
            spacing: Theme.spacingXS

            Row {
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.spacingXS

                DankIcon {
                    name: "book_2"
                    size: Theme.iconSizeSmall
                    color: Theme.surfaceText
                    anchors.verticalCenter: parent.verticalCenter
                }

                Rectangle {
                    width: grammarLabel.implicitWidth + Theme.spacingXS
                    color: "transparent"
                    height: parent.height
                    anchors.verticalCenter: parent.verticalCenter

                    StyledText {
                        id: grammarLabel
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.horizontalCenter: parent.horizontalCenter
                        text: root.bunpro.current.grammar.total
                        font.pixelSize: Theme.fontSizeSmall
                    }
                }
            }

            Row {
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.spacingXS

                DankIcon {
                    name: "language_japanese_kana"
                    size: Theme.iconSize - 4
                    color: Theme.surfaceText
                    anchors.verticalCenter: parent.verticalCenter
                }

                Rectangle {
                    width: vocabLabel.implicitWidth + Theme.spacingXS
                    color: "transparent"
                    height: parent.height
                    anchors.verticalCenter: parent.verticalCenter

                    StyledText {
                        id: vocabLabel
                        anchors.verticalCenter: parent.verticalCenter
                        anchors.horizontalCenter: parent.horizontalCenter
                        text: root.bunpro.current.vocab.total
                        font.pixelSize: Theme.fontSizeSmall
                    }
                }
            }
        }
    }
}
