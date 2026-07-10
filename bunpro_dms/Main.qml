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
                        text: BunproService.current.grammar.total
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
                        text: BunproService.current.vocab.total
                        font.pixelSize: Theme.fontSizeSmall
                    }
                }
            }
        }
    }
}
