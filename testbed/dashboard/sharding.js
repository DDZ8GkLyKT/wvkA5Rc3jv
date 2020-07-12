/* global _ */

/*
 * Complex scripted dashboard
 * This script generates a dashboard object that Grafana can load. It also takes a number of user
 * supplied URL parameters (in the ARGS variable)
 *
 * Return a dashboard object, or a function
 *
 * For async scripts, return a function, this function must take a single callback function as argument,
 * call this callback function with the dashboard object (look at scripted_async.js for an example)
 */

'use strict';

// accessible variables in this scope
var window, document, ARGS, $, jQuery, moment, kbn;

// Setup some variables
var dashboard;

// All url parameters are available via the ARGS object
var ARGS;

// Initialize a skeleton with nothing but a rows array and service object
dashboard = {
    panels: [],
};

// Set a title
dashboard.title = 'Sharding';

// Set default time
// time can be overridden in the url using from/to parameters, but this is
// handled automatically in grafana core during dashboard initialization
dashboard.time = {
    from: "now-5m",
    to: "now"
};

dashboard.refresh = '1s';

dashboard.timepicker = {
    "refresh_intervals": [
        "1s",
        "3s",
        "5s",
        "10s",
        "30s",
        "1m"
    ],
    "time_options": [
        "1m",
        "5m",
        "10m",
        "30m",
        "1h"
    ]
};

var nodes = 1;
if(!_.isUndefined(ARGS.nodes)) {
    nodes = parseInt(ARGS.nodes, 10);
}

// prepare the per-node transaction rate queries
var txRateTargets;
txRateTargets = [];
for (var i = 0; i < nodes; i++) {
    txRateTargets.push({
        "target": "node_" + i.toString() + ":confirmed_tx",
        "type": "timeserie"
    });
}

// transaction rate row
dashboard.panels.push(
    {
        "cacheTimeout": null,
        "colorBackground": false,
        "colorValue": false,
        "colors": [
            "#299c46",
            "rgba(237, 129, 40, 0.89)",
            "#d44a3a"
        ],
        "datasource": "Sharding",
        "format": "short",
        "gauge": {
            "maxValue": 100,
            "minValue": 0,
            "show": false,
            "thresholdLabels": false,
            "thresholdMarkers": true
        },
        "gridPos": {
            "h": 2,
            "w": 11,
            "x": 0,
            "y": 0
        },
        "id": 2,
        "interval": null,
        "links": [],
        "mappingType": 1,
        "mappingTypes": [
            {
                "name": "value to text",
                "value": 1
            },
            {
                "name": "range to text",
                "value": 2
            }
        ],
        "maxDataPoints": 100,
        "nullPointMode": "connected",
        "nullText": null,
        "options": {},
        "postfix": " transactions",
        "postfixFontSize": "50%",
        "prefix": "",
        "prefixFontSize": "50%",
        "rangeMaps": [
            {
                "from": "null",
                "text": "N/A",
                "to": "null"
            }
        ],
        "sparkline": {
            "fillColor": "rgba(31, 118, 189, 0.18)",
            "full": false,
            "lineColor": "rgb(31, 120, 193)",
            "show": false
        },
        "tableColumn": "",
        "targets": [
            {
                "refId": "A",
                "target": "accumulative:confirmed_tx",
                "type": "timeserie"
            }
        ],
        "thresholds": "",
        "timeFrom": null,
        "timeShift": null,
        "title": "Total confirmed transactions",
        "type": "singlestat",
        "valueFontSize": "80%",
        "valueMaps": [
            {
                "op": "=",
                "text": "N/A",
                "value": "null"
            }
        ],
        "valueName": "current"
    },
    {
        "aliasColors": {},
        "bars": false,
        "dashLength": 10,
        "dashes": false,
        "datasource": "Sharding",
        "fill": 0,
        "gridPos": {
            "h": 10,
            "w": 12,
            "x": 11,
            "y": 0
        },
        "id": 26,
        "legend": {
            "avg": false,
            "current": false,
            "max": false,
            "min": false,
            "show": false,
            "total": false,
            "values": false
        },
        "lines": true,
        "linewidth": 2,
        "links": [],
        "nullPointMode": "null",
        "options": {},
        "percentage": false,
        "pointradius": 2,
        "points": false,
        "renderer": "flot",
        "seriesOverrides": [],
        "spaceLength": 10,
        "stack": false,
        "steppedLine": false,
        "targets": [
            {
                "refId": "A",
                "target": "node_0:confirmed_tx",
                "type": "timeserie"
            },
            {
                "refId": "B",
                "target": "node_2:confirmed_tx",
                "type": "timeserie"
            },
            {
                "refId": "C",
                "target": "node_4:confirmed_tx",
                "type": "timeserie"
            },
            {
                "refId": "D",
                "target": "node_6:confirmed_tx",
                "type": "timeserie"
            },
            {
                "refId": "E",
                "target": "node_8:confirmed_tx",
                "type": "timeserie"
            },
            {
                "refId": "F",
                "target": "node_10:confirmed_tx",
                "type": "timeserie"
            }
        ],
        "thresholds": [],
        "timeFrom": null,
        "timeRegions": [],
        "timeShift": null,
        "title": "Throughput per shard",
        "tooltip": {
            "shared": true,
            "sort": 0,
            "value_type": "individual"
        },
        "type": "graph",
        "xaxis": {
            "buckets": null,
            "mode": "time",
            "name": null,
            "show": true,
            "values": []
        },
        "yaxes": [
            {
                "format": "short",
                "label": "Transactions",
                "logBase": 1,
                "max": null,
                "min": "0",
                "show": true
            },
            {
                "format": "short",
                "label": null,
                "logBase": 1,
                "max": null,
                "min": "0",
                "show": true
            }
        ],
        "yaxis": {
            "align": false,
            "alignLevel": null
        }
    },
    {
        "datasource": "Sharding",
        "gridPos": {
            "h": 8,
            "w": 5,
            "x": 0,
            "y": 2
        },
        "id": 11,
        "links": [],
        "options": {
            "fieldOptions": {
                "calcs": [
                    "mean"
                ],
                "defaults": {
                    "max": 1000000,
                    "min": 0
                },
                "mappings": [],
                "override": {},
                "thresholds": [
                    {
                        "color": "green",
                        "index": 0,
                        "value": null
                    }
                ],
                "values": false
            },
            "orientation": "auto",
            "showThresholdLabels": false,
            "showThresholdMarkers": true
        },
        "pluginVersion": "6.2.5",
        "targets": [
            {
                "refId": "A",
                "target": "average:confirmed_tx",
                "type": "timeserie"
            }
        ],
        "timeFrom": null,
        "timeShift": null,
        "title": "Total Throughput",
        "type": "gauge"
    },
    {
        "datasource": "Sharding",
        "gridPos": {
            "h": 4,
            "w": 2,
            "x": 5,
            "y": 2
        },
        "id": 15,
        "links": [],
        "options": {
            "fieldOptions": {
                "calcs": [
                    "mean"
                ],
                "defaults": {
                    "max": 1000000,
                    "min": 0
                },
                "mappings": [],
                "override": {},
                "thresholds": [
                    {
                        "color": "green",
                        "index": 0,
                        "value": null
                    }
                ],
                "values": false
            },
            "orientation": "auto",
            "showThresholdLabels": false,
            "showThresholdMarkers": true
        },
        "pluginVersion": "6.2.5",
        "targets": [
            {
                "refId": "A",
                "target": "node_0_shard:confirmed_tx",
                "type": "timeserie"
            }
        ],
        "timeFrom": null,
        "timeShift": null,
        "title": "Shard 1",
        "type": "gauge"
    },
    {
        "datasource": "Sharding",
        "gridPos": {
            "h": 4,
            "w": 2,
            "x": 7,
            "y": 2
        },
        "id": 14,
        "links": [],
        "options": {
            "fieldOptions": {
                "calcs": [
                    "mean"
                ],
                "defaults": {
                    "max": 1000000,
                    "min": 0
                },
                "mappings": [],
                "override": {},
                "thresholds": [
                    {
                        "color": "green",
                        "index": 0,
                        "value": null
                    }
                ],
                "values": false
            },
            "orientation": "auto",
            "showThresholdLabels": false,
            "showThresholdMarkers": true
        },
        "pluginVersion": "6.2.5",
        "targets": [
            {
                "refId": "A",
                "target": "node_2_shard:confirmed_tx",
                "type": "timeserie"
            }
        ],
        "timeFrom": null,
        "timeShift": null,
        "title": "Shard 2",
        "type": "gauge"
    },
    {
        "datasource": "Sharding",
        "gridPos": {
            "h": 4,
            "w": 2,
            "x": 9,
            "y": 2
        },
        "id": 24,
        "links": [],
        "options": {
            "fieldOptions": {
                "calcs": [
                    "mean"
                ],
                "defaults": {
                    "max": 1000000,
                    "min": 0
                },
                "mappings": [],
                "override": {},
                "thresholds": [
                    {
                        "color": "green",
                        "index": 0,
                        "value": null
                    }
                ],
                "values": false
            },
            "orientation": "auto",
            "showThresholdLabels": false,
            "showThresholdMarkers": true
        },
        "pluginVersion": "6.2.5",
        "targets": [
            {
                "refId": "A",
                "target": "node_4:confirmed_tx",
                "type": "timeserie"
            }
        ],
        "timeFrom": null,
        "timeShift": null,
        "title": "Shard 3",
        "type": "gauge"
    },
    {
        "datasource": "Sharding",
        "gridPos": {
            "h": 4,
            "w": 2,
            "x": 5,
            "y": 6
        },
        "id": 12,
        "links": [],
        "options": {
            "fieldOptions": {
                "calcs": [
                    "mean"
                ],
                "defaults": {
                    "max": 1000000,
                    "min": 0
                },
                "mappings": [],
                "override": {},
                "thresholds": [
                    {
                        "color": "green",
                        "index": 0,
                        "value": null
                    }
                ],
                "values": false
            },
            "orientation": "auto",
            "showThresholdLabels": false,
            "showThresholdMarkers": true
        },
        "pluginVersion": "6.2.5",
        "targets": [
            {
                "refId": "A",
                "target": "node_6_shard:confirmed_tx",
                "type": "timeserie"
            }
        ],
        "timeFrom": null,
        "timeShift": null,
        "title": "Shard 4",
        "type": "gauge"
    },
    {
        "datasource": "Sharding",
        "gridPos": {
            "h": 4,
            "w": 2,
            "x": 7,
            "y": 6
        },
        "id": 17,
        "links": [],
        "options": {
            "fieldOptions": {
                "calcs": [
                    "mean"
                ],
                "defaults": {
                    "max": 1000000,
                    "min": 0
                },
                "mappings": [],
                "override": {},
                "thresholds": [
                    {
                        "color": "green",
                        "index": 0,
                        "value": null
                    }
                ],
                "values": false
            },
            "orientation": "auto",
            "showThresholdLabels": false,
            "showThresholdMarkers": true
        },
        "pluginVersion": "6.2.5",
        "targets": [
            {
                "refId": "A",
                "target": "node_8_shard:confirmed_tx",
                "type": "timeserie"
            }
        ],
        "timeFrom": null,
        "timeShift": null,
        "title": "Shard 5",
        "type": "gauge"
    },
    {
        "datasource": "Sharding",
        "gridPos": {
            "h": 4,
            "w": 2,
            "x": 9,
            "y": 6
        },
        "id": 23,
        "links": [],
        "options": {
            "fieldOptions": {
                "calcs": [
                    "mean"
                ],
                "defaults": {
                    "max": 1000000,
                    "min": 0
                },
                "mappings": [],
                "override": {},
                "thresholds": [
                    {
                        "color": "green",
                        "index": 0,
                        "value": null
                    }
                ],
                "values": false
            },
            "orientation": "auto",
            "showThresholdLabels": false,
            "showThresholdMarkers": true
        },
        "pluginVersion": "6.2.5",
        "targets": [
            {
                "refId": "A",
                "target": "node_10_shard:confirmed_tx",
                "type": "timeserie"
            }
        ],
        "timeFrom": null,
        "timeShift": null,
        "title": "Shard 6",
        "type": "gauge"
    },
    {
        "datasource": "Sharding",
        "gridPos": {
            "h": 8,
            "w": 5,
            "x": 0,
            "y": 10
        },
        "id": 18,
        "links": [],
        "options": {
            "fieldOptions": {
                "calcs": [
                    "mean"
                ],
                "defaults": {
                    "max": 1000000,
                    "min": 0
                },
                "mappings": [],
                "override": {},
                "thresholds": [
                    {
                        "color": "green",
                        "index": 0,
                        "value": null
                    }
                ],
                "values": false
            },
            "orientation": "auto",
            "showThresholdLabels": false,
            "showThresholdMarkers": true
        },
        "pluginVersion": "6.2.5",
        "targets": [
            {
                "refId": "A",
                "target": "accumulative:txblk_cfm_mean",
                "type": "timeserie"
            }
        ],
        "timeFrom": null,
        "timeShift": null,
        "title": "Average Confirmation Latency",
        "type": "gauge"
    },
    {
        "datasource": "Sharding",
        "gridPos": {
            "h": 4,
            "w": 2,
            "x": 5,
            "y": 10
        },
        "id": 21,
        "links": [],
        "options": {
            "fieldOptions": {
                "calcs": [
                    "mean"
                ],
                "defaults": {
                    "max": 1000000,
                    "min": 0
                },
                "mappings": [],
                "override": {},
                "thresholds": [
                    {
                        "color": "green",
                        "index": 0,
                        "value": null
                    }
                ],
                "values": false
            },
            "orientation": "auto",
            "showThresholdLabels": false,
            "showThresholdMarkers": true
        },
        "pluginVersion": "6.2.5",
        "targets": [
            {
                "refId": "A",
                "target": "node_0_shard:txblk_cfm_mean",
                "type": "timeserie"
            }
        ],
        "timeFrom": null,
        "timeShift": null,
        "title": "Shard 1",
        "type": "gauge"
    },
    {
        "datasource": "Sharding",
        "gridPos": {
            "h": 4,
            "w": 2,
            "x": 7,
            "y": 10
        },
        "id": 20,
        "links": [],
        "options": {
            "fieldOptions": {
                "calcs": [
                    "mean"
                ],
                "defaults": {
                    "max": 1000000,
                    "min": 0
                },
                "mappings": [],
                "override": {},
                "thresholds": [
                    {
                        "color": "green",
                        "index": 0,
                        "value": null
                    }
                ],
                "values": false
            },
            "orientation": "auto",
            "showThresholdLabels": false,
            "showThresholdMarkers": true
        },
        "pluginVersion": "6.2.5",
        "targets": [
            {
                "refId": "A",
                "target": "node_2_shard:txblk_cfm_mean",
                "type": "timeserie"
            }
        ],
        "timeFrom": null,
        "timeShift": null,
        "title": "Shard 2",
        "type": "gauge"
    },
    {
        "datasource": "Sharding",
        "gridPos": {
            "h": 4,
            "w": 2,
            "x": 9,
            "y": 10
        },
        "id": 22,
        "links": [],
        "options": {
            "fieldOptions": {
                "calcs": [
                    "mean"
                ],
                "defaults": {
                    "max": 1000000,
                    "min": 0
                },
                "mappings": [],
                "override": {},
                "thresholds": [
                    {
                        "color": "green",
                        "index": 0,
                        "value": null
                    }
                ],
                "values": false
            },
            "orientation": "auto",
            "showThresholdLabels": false,
            "showThresholdMarkers": true
        },
        "pluginVersion": "6.2.5",
        "targets": [
            {
                "refId": "A",
                "target": "node_4_shard:txblk_cfm_mean",
                "type": "timeserie"
            }
        ],
        "timeFrom": null,
        "timeShift": null,
        "title": "Shard 3",
        "type": "gauge"
    },
    {
        "aliasColors": {},
        "bars": false,
        "dashLength": 10,
        "dashes": false,
        "datasource": "Sharding",
        "fill": 1,
        "gridPos": {
            "h": 8,
            "w": 12,
            "x": 11,
            "y": 10
        },
        "id": 27,
        "legend": {
            "avg": false,
            "current": false,
            "max": false,
            "min": false,
            "show": false,
            "total": false,
            "values": false
        },
        "lines": true,
        "linewidth": 1,
        "links": [],
        "nullPointMode": "null",
        "options": {},
        "percentage": false,
        "pointradius": 2,
        "points": false,
        "renderer": "flot",
        "seriesOverrides": [],
        "spaceLength": 10,
        "stack": false,
        "steppedLine": false,
        "targets": [
            {
                "refId": "A",
                "target": "node_0_shard:txblk_cfm_mean",
                "type": "timeserie"
            },
            {
                "refId": "B",
                "target": "node_2_shard:txblk_cfm_mean",
                "type": "timeserie"
            },
            {
                "refId": "C",
                "target": "node_4_shard:txblk_cfm_mean",
                "type": "timeserie"
            },
            {
                "refId": "D",
                "target": "node_6_shard:txblk_cfm_mean",
                "type": "timeserie"
            },
            {
                "refId": "E",
                "target": "node_8_shard:txblk_cfm_mean",
                "type": "timeserie"
            },
            {
                "refId": "F",
                "target": "node_10_shard:txblk_cfm_mean",
                "type": "timeserie"
            }
        ],
        "thresholds": [],
        "timeFrom": null,
        "timeRegions": [],
        "timeShift": null,
        "title": "Confirmation Latency per shard",
        "tooltip": {
            "shared": true,
            "sort": 0,
            "value_type": "individual"
        },
        "type": "graph",
        "xaxis": {
            "buckets": null,
            "mode": "time",
            "name": null,
            "show": true,
            "values": []
        },
        "yaxes": [
            {
                "format": "s",
                "label": "Seconds",
                "logBase": 1,
                "max": null,
                "min": "0",
                "show": true
            },
            {
                "format": "short",
                "label": null,
                "logBase": 1,
                "max": null,
                "min": null,
                "show": true
            }
        ],
        "yaxis": {
            "align": false,
            "alignLevel": null
        }
    },
    {
        "datasource": "Sharding",
        "gridPos": {
            "h": 4,
            "w": 2,
            "x": 5,
            "y": 14
        },
        "id": 13,
        "links": [],
        "options": {
            "fieldOptions": {
                "calcs": [
                    "mean"
                ],
                "defaults": {
                    "max": 1000000,
                    "min": 0
                },
                "mappings": [],
                "override": {},
                "thresholds": [
                    {
                        "color": "green",
                        "index": 0,
                        "value": null
                    }
                ],
                "values": false
            },
            "orientation": "auto",
            "showThresholdLabels": false,
            "showThresholdMarkers": true
        },
        "pluginVersion": "6.2.5",
        "targets": [
            {
                "refId": "A",
                "target": "node_6_shard:txblk_cfm_mean",
                "type": "timeserie"
            }
        ],
        "timeFrom": null,
        "timeShift": null,
        "title": "Shard 4",
        "type": "gauge"
    },
    {
        "datasource": "Sharding",
        "gridPos": {
            "h": 4,
            "w": 2,
            "x": 7,
            "y": 14
        },
        "id": 16,
        "links": [],
        "options": {
            "fieldOptions": {
                "calcs": [
                    "mean"
                ],
                "defaults": {
                    "max": 1000000,
                    "min": 0
                },
                "mappings": [],
                "override": {},
                "thresholds": [
                    {
                        "color": "green",
                        "index": 0,
                        "value": null
                    }
                ],
                "values": false
            },
            "orientation": "auto",
            "showThresholdLabels": false,
            "showThresholdMarkers": true
        },
        "pluginVersion": "6.2.5",
        "targets": [
            {
                "refId": "A",
                "target": "node_8_shard:txblk_cfm_mean",
                "type": "timeserie"
            }
        ],
        "timeFrom": null,
        "timeShift": null,
        "title": "Shard 5",
        "type": "gauge"
    },
    {
        "datasource": "Sharding",
        "gridPos": {
            "h": 4,
            "w": 2,
            "x": 9,
            "y": 14
        },
        "id": 19,
        "links": [],
        "options": {
            "fieldOptions": {
                "calcs": [
                    "mean"
                ],
                "defaults": {
                    "max": 1000000,
                    "min": 0
                },
                "mappings": [],
                "override": {},
                "thresholds": [
                    {
                        "color": "green",
                        "index": 0,
                        "value": null
                    }
                ],
                "values": false
            },
            "orientation": "auto",
            "showThresholdLabels": false,
            "showThresholdMarkers": true
        },
        "pluginVersion": "6.2.5",
        "targets": [
            {
                "refId": "A",
                "target": "node_10_shard:txblk_cfm_mean",
                "type": "timeserie"
            }
        ],
        "timeFrom": null,
        "timeShift": null,
        "title": "Shard 6",
        "type": "gauge"
    }
);

return dashboard;