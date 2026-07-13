pragma ComponentBehavior: Bound

import QtQuick
import QtQuick.Layouts

import qs.Common
import qs.Widgets
import qs.Modules.Plugins
import qs.Services

import "./Services"

PluginComponent {
    id: root

    layerNamespacePlugin: "bunpro"

    Connections {
        target: root.pluginService

        function onPluginDataChanged(changedId) {
            if (changedId !== root.layerNamespacePlugin) {
                return;
            }

            const oldKey = BunproService.apiKey;
            const newKey = root.pluginService.loadPluginData(root.layerNamespacePlugin, "apiKey", "");
            const oldDangerousLoad = BunproService.dangerouslyAuthenticateUsingApiKey;
            const newDangerousLoad = root.pluginService.loadPluginData(root.layerNamespacePlugin, "dangerouslyAuthenticateUsingApiKey", true);
            // settings changed, so we should try to reload data
            const shouldReload = (oldKey != newKey) || (oldDangerousLoad !== newDangerousLoad);

            const updateInterval = root.pluginService.loadPluginData(root.layerNamespacePlugin, "updateInterval", 15);

            BunproService.apiKey = newKey;
            BunproService.dangerouslyAuthenticateUsingApiKey = newDangerousLoad;
            BunproService.updateInterval = updateInterval;

            if (shouldReload) {
                BunproService.refreshForecast();
            }
        }

        function onPluginLoaded(pluginId) {
            if (pluginId !== root.layerNamespacePlugin) {
                return;
            }

            // now set settings and refresh
            BunproService.apiKey = root.pluginData?.apiKey || "";
            BunproService.dangerouslyAuthenticateUsingApiKey = root.pluginData?.dangerouslyAuthenticateUsingApiKey ?? true;
            BunproService.updateInterval = root.pluginData?.updateInterval ?? 15;

            BunproService.refreshForecast();
        }
    }

    pillRightClickAction: (x, y, width, section, screen) => BunproService.refreshForecast()

    horizontalBarPill: Component {
        Row {
            spacing: Theme.spacingS

            Row {
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.spacingXS

                StyledText {
                    anchors.verticalCenter: parent.verticalCenter
                    text: "文"
                    font.pixelSize: Theme.fontSizeMedium
                    font.family: "Open Sans Condensed"
                }

                Rectangle {
                    width: grammarLabel.implicitWidth
                    color: "transparent"
                    height: parent.height
                    anchors.verticalCenter: parent.verticalCenter

                    StyledText {
                        id: grammarLabel
                        anchors.centerIn: parent
                        anchors.verticalCenterOffset: 1.0
                        text: BunproService.current.grammar.total
                        font.pixelSize: Theme.fontSizeSmall
                    }
                }
            }

            Row {
                anchors.verticalCenter: parent.verticalCenter
                spacing: Theme.spacingXS

                DankIcon {
                    anchors.verticalCenter: parent.verticalCenter
                    anchors.verticalCenterOffset: 0.5
                    name: "language_japanese_kana"
                    size: Theme.iconSize - 4
                    color: Theme.surfaceText
                }

                Rectangle {
                    width: vocabLabel.implicitWidth
                    color: "transparent"
                    height: parent.height
                    anchors.verticalCenter: parent.verticalCenter

                    StyledText {
                        id: vocabLabel
                        anchors.centerIn: parent
                        anchors.verticalCenterOffset: 1.0
                        text: BunproService.current.vocab.total
                        font.pixelSize: Theme.fontSizeSmall
                    }
                }
            }
        }
    }
}
